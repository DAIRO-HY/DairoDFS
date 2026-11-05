use crate::dao::storage_recycle_dao::StorageRecycle;
use crate::dao::storage_file_dao::StorageFile;
use crate::dao::{dfs_file_dao, dfs_file_delete_dao, sql_log_dao, storage_file_dao, storage_recycle_dao};
use crate::extension::number::NumberExtension;
use crate::service::dfs_file_delete_service;
use crate::service::sync;
use crate::service::sync::sync_info;
use crate::util::{db_backup_util, time_util};
use crate::web::biz_error::BizError;
use crate::{db, system_config};
use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::Duration;

/// 一天的毫秒数
const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// 同步日志保留天数
const SQL_LOG_KEEP_DAYS: i64 = 100;

/// 一次要回收的文件超过这个数量时暂停,需要管理员确认之后才继续
const BLOCK_COUNT: i64 = 500;

/// 一次要回收的文件超过存储文件总数的这个百分比时暂停,需要管理员确认之后才继续
const BLOCK_PERCENT: i64 = 1;

/// 按百分比判断时的最少数量(文件总数很少时,1%只有几个文件,不按百分比判断)
const BLOCK_PERCENT_MIN_COUNT: i64 = 50;

/// 隔离区文件夹名,放在存储目录下,和存储文件在同一块磁盘上,移入隔离区只需要改名
const RECYCLE_DIR_NAME: &str = ".dfs-recycle";

/// 回收定时器的状态
static STATE: Mutex<TimerState> = Mutex::new(TimerState {
    is_running: false,
    error: String::new(),
    warning: String::new(),
    last_run_time: 0,
    blocked: None,
});

/// 回收定时器的状态
#[derive(Debug, Clone)]
pub struct TimerState {

    // 是否正在运行
    pub is_running: bool,

    // 上次执行的错误信息
    pub error: String,

    // 需要管理员注意的信息,例如自动恢复了被误回收的文件
    pub warning: String,

    // 上次执行完成的时间
    pub last_run_time: i64,

    // 因为数量异常暂停的回收,等待管理员确认
    pub blocked: Option<Blocked>,
}

/// 因为数量异常暂停的回收
#[derive(Debug, Clone)]
pub struct Blocked {

    // 本次需要回收的文件数量
    pub count: i64,

    // 存储文件总数
    pub total: i64,

    // 只回收这个时间之前创建的存储文件
    pub time: i64,

    // 发现时间
    pub date: i64,
}

/// 获取回收定时器的状态
pub fn state() -> TimerState {
    STATE.lock().unwrap().clone()
}

/// 启动回收定时器,每天凌晨3点执行
pub fn start() {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(until_next_run()).await;
            run().await;
        }
    });
}

/// 距离下一次凌晨3点的时间
fn until_next_run() -> Duration {
    let now = Local::now();
    let today = now.date_naive().and_time(NaiveTime::from_hms_opt(3, 0, 0).unwrap());
    let next = if now.naive_local() < today { today } else { today + ChronoDuration::days(1) };
    (next - now.naive_local()).to_std().unwrap_or(Duration::from_secs(60))
}

/// 执行一次回收(正在执行时不重复执行)
pub async fn run() {
    exclusive(run_inner()).await;
}

/// 立即回收没有被使用的存储文件(高级功能页面使用)
/// - time 只回收这个时间之前创建的存储文件,避免正在上传的文件被回收
pub async fn recycle_now(time: i64) {
    exclusive(async move { recycle_not_use_storage(time, None).await.map(|_| ()) }).await;
}

/// 管理员确认之后,继续执行因为数量异常暂停的回收
pub async fn confirm_blocked() {
    let Some(blocked) = state().blocked else {
        return;
    };
    exclusive(async move { recycle_not_use_storage(blocked.time, Some(blocked.count)).await.map(|_| ()) }).await;
}

/// 立即检查并恢复隔离区里仍然被使用的文件(高级功能页面使用)
pub async fn restore_used_now() {
    exclusive(restore_used()).await;
}

/// 同一时间只执行一个回收任务,并记录执行结果
async fn exclusive(task: impl Future<Output = Result<(), String>>) {
    {
        let mut state = STATE.lock().unwrap();
        if state.is_running {
            return;
        }
        state.is_running = true;
    }
    let result = task.await;
    let mut state = STATE.lock().unwrap();
    state.is_running = false;
    state.error = result.err().unwrap_or_default();
    state.last_run_time = time_util::current_millis();
}

async fn run_inner() -> Result<(), String> {
    db_backup_util::backup().await?;
    delete_sql_log().await?;
    delete_trash_timeout().await.map_err(|e| e.to_string())?;

    //先恢复隔离区里仍然被使用的文件(正常情况下不会有),再回收
    restore_used().await?;
    delete_timeout_file_delete().await?;
    let time = time_util::current_millis() - delete_storage_timeout() * DAY_MILLIS;
    recycle_not_use_storage(time, None).await?;
    purge_recycle().await
}

/// 回收站保留天数,至少1天,避免配置错误时立即删除
fn trash_timeout() -> i64 {
    system_config::get().trash_timeout.max(1)
}

/// 存储空间回收间隔天数,至少1天,避免配置错误时把正在上传的文件当成没有被使用
fn delete_storage_timeout() -> i64 {
    system_config::get().delete_storage_timeout.max(1)
}

/// 隔离区保留天数,至少1天
fn recycle_keep_days() -> i64 {
    system_config::get().recycle_keep_days.max(1)
}

/// 删除超出指定时间的同步日志
async fn delete_sql_log() -> Result<(), String> {
    let time = time_util::current_millis() - SQL_LOG_KEEP_DAYS * DAY_MILLIS;
    sql_log_dao::delete_before(&mut lib_db::get_context(), time).await.map_err(|e| e.to_string())?;

    //超过这个时间的修改不会再同步过来,修改时间也不需要了
    sync::version::delete_before(&mut lib_db::get_context(), time).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 彻底删除垃圾箱里到期的文件(只是把文件记录移到彻底删除表,存储文件之后再回收)
/// 这一步会记录同步日志,分机跟着主机同步,所以只读模式下不执行
async fn delete_trash_timeout() -> Result<(), BizError> {
    if system_config::get().is_read_only {
        return Ok(());
    }
    let mut pre_ids = Vec::new();
    loop {
        let time = time_util::current_millis() - trash_timeout() * DAY_MILLIS;
        let ids = dfs_file_dao::select_ids_by_delete_timeout(&mut lib_db::get_context(), time).await?;
        if ids.is_empty() || ids == pre_ids {
            //没有数据了,或者这一批数据删除不掉,避免死循环
            return Ok(());
        }
        let mut ctx = db::begin().await?;
        dfs_file_delete_service::delete_by_ids(&mut ctx, &ids).await?;
        db::commit(ctx).await?;
        pre_ids = ids;
    }
}

/// 删除彻底删除表里到期的记录,对应的存储文件没有被使用时,之后由recycle_not_use_storage移到隔离区
/// 只删除本机的记录,不记录同步日志,每台机器各自回收
async fn delete_timeout_file_delete() -> Result<(), String> {
    let mut pre_ids = Vec::new();
    loop {
        let time = time_util::current_millis() - delete_storage_timeout() * DAY_MILLIS;
        let mut ctx = lib_db::get_context();
        let list = dfs_file_delete_dao::select_by_timeout(&mut ctx, time).await.map_err(|e| e.to_string())?;
        let ids: Vec<i64> = list.iter().map(|it| it.id).collect();
        if ids.is_empty() || ids == pre_ids {
            return Ok(());
        }
        for id in &ids {
            dfs_file_delete_dao::delete(&mut ctx, *id).await.map_err(|e| e.to_string())?;
        }
        pre_ids = ids;
    }
}

/// 检查现在能不能回收存储文件
/// 文件记录不完整时(例如分机正在全量同步),还没同步过来的文件会被当成没有被使用,所以不能回收
async fn check_safe() -> Result<(), String> {
    if sync::IS_TABLE_SYNCING.load(Ordering::Relaxed) {
        return Err("正在全量同步,暂不回收存储文件".to_string());
    }
    if sync::is_full_sync_pending() || !sync::all_synced() {
        return Err("全量同步还没有完成,暂不回收存储文件".to_string());
    }
    if sync_info::has_error() {
        return Err("数据同步出错,本机数据可能不完整,暂不回收存储文件".to_string());
    }
    let pool = lib_db::get();
    let file_count: i64 = sqlx::query_scalar("select count(*) from dfs_file").fetch_one(&pool).await.map_err(|e| e.to_string())?;
    let storage_count: i64 = sqlx::query_scalar("select count(*) from storage_file").fetch_one(&pool).await.map_err(|e| e.to_string())?;
    if file_count == 0 && storage_count > 0 {
        return Err("文件记录为空,但存在存储文件,数据可能不完整,暂不回收存储文件".to_string());
    }
    Ok(())
}

/// 要回收的文件数量是否异常
fn is_abnormal(count: i64, total: i64) -> bool {
    count > BLOCK_COUNT || (count > BLOCK_PERCENT_MIN_COUNT && count * 100 > total * BLOCK_PERCENT)
}

/// 把没有被使用的存储文件移到隔离区
/// - time 只回收这个时间之前创建的存储文件,避免正在上传的文件被回收
/// - confirmed_count 管理员确认过的数量,数量比确认时多时重新暂停;为None时按正常规则判断数量是否异常
/// - return 移入隔离区的数量
async fn recycle_not_use_storage(time: i64, confirmed_count: Option<i64>) -> Result<i64, String> {
    check_safe().await?;
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    let count = storage_file_dao::count_not_use(&mut ctx, time).await.map_err(err)?;
    let total = storage_file_dao::count_all(&mut ctx).await.map_err(err)?;
    let need_block = match confirmed_count {
        Some(confirmed) => count > confirmed,
        None => is_abnormal(count, total),
    };
    if need_block {
        STATE.lock().unwrap().blocked = Some(Blocked {
            count,
            total,
            time,
            date: time_util::current_millis(),
        });
        return Err(format!(
            "本次需要回收{}个存储文件(共{}个),数量异常,已暂停回收,请在高级功能页面确认后继续",
            count, total
        ));
    }
    STATE.lock().unwrap().blocked = None;
    let mut moved = 0;
    let mut pre_ids = Vec::new();
    loop {
        let ids = storage_file_dao::select_not_use_ids(&mut ctx, time).await.map_err(err)?;
        if ids.is_empty() || ids == pre_ids {
            return Ok(moved);
        }
        for id in &ids {
            if move_to_recycle(*id).await? {
                moved += 1;
            }
        }
        pre_ids = ids;
    }
}

/// 存储文件在隔离区里的路径:存储目录/.dfs-recycle/日期/文件名
/// 存储文件的路径是 存储目录/年-月/文件名,隔离区放在存储目录下,保证和存储文件在同一块磁盘上
fn recycle_path(path: &Path) -> Result<PathBuf, String> {
    let (Some(name), Some(storage_dir)) = (path.file_name(), path.parent().and_then(|it| it.parent())) else {
        return Err(format!("存储文件路径不正确:{}", path.display()));
    };
    Ok(storage_dir
        .join(RECYCLE_DIR_NAME)
        .join(Local::now().format("%Y-%m-%d").to_string())
        .join(name))
}

/// 把一个没有被使用的存储文件移到隔离区
/// 在同一个事务里:确认没有被使用并删除存储记录 -> 移动文件 -> 添加隔离区记录,任何一步失败都会回滚,文件不会丢失
/// - return 是否移入了隔离区(已经被使用时返回false)
async fn move_to_recycle(id: i64) -> Result<bool, String> {
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    let storage = match storage_file_dao::select_one(&mut ctx, id).await {
        Ok(it) => it,
        Err(sqlx::Error::RowNotFound) => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let path = PathBuf::from(&storage.path);
    let target = recycle_path(&path)?;
    ctx.begin_immediate().await.map_err(err)?;
    if storage_file_dao::delete_if_not_use(&mut ctx, id).await.map_err(err)? == 0 {
        //判断之后被使用了
        ctx.rollback().await.map_err(err)?;
        return Ok(false);
    }
    let (recycle_path, size) = match tokio::fs::metadata(&path).await {
        Ok(meta) => {
            let moved = async {
                tokio::fs::create_dir_all(target.parent().unwrap()).await?;
                if tokio::fs::try_exists(&target).await? {
                    return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "隔离区已经存在同名文件"));
                }
                tokio::fs::rename(&path, &target).await
            };
            if let Err(e) = moved.await {
                ctx.rollback().await.map_err(err)?;
                return Err(format!("文件{}移入隔离区失败:{}", path.display(), e));
            }
            (Some(target.to_string_lossy().to_string()), meta.len() as i64)
        }

        //文件已经不存在,只删除记录
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (None, 0),
        Err(e) => {
            ctx.rollback().await.map_err(err)?;
            return Err(format!("读取文件{}失败:{}", path.display(), e));
        }
    };
    let record = StorageRecycle {
        id,
        md5: storage.md5,
        path: storage.path,
        recyclePath: recycle_path.clone(),
        size,
        date: time_util::current_millis(),
    };
    let result = async {
        storage_recycle_dao::insert(&mut ctx, &record).await?;
        ctx.commit().await
    }
    .await;
    if let Err(e) = result {
        //记录没有保存成功,把文件移回原来的位置
        if let Some(recycle_path) = &recycle_path {
            let _ = tokio::fs::rename(recycle_path, &path).await;
        }
        return Err(format!("文件{}移入隔离区失败:{}", path.display(), e));
    }
    Ok(true)
}

/// 恢复隔离区里仍然被使用的文件(说明被误回收,正常情况下不会有)
async fn restore_used() -> Result<(), String> {
    let list = storage_recycle_dao::select_used(&mut lib_db::get_context()).await.map_err(|e| e.to_string())?;
    if list.is_empty() {
        return Ok(());
    }
    let mut restored = 0;
    let mut errors = Vec::new();
    for it in &list {
        match restore(it).await {
            Ok(_) => restored += 1,
            Err(e) => errors.push(e),
        }
    }
    let mut warning = format!(
        "{}发现{}个被回收的存储文件仍然被使用,已自动恢复{}个",
        time_util::current_millis().date_format(),
        list.len(),
        restored
    );
    if !errors.is_empty() {
        warning += &format!(",恢复失败:{}", errors.join(";"));
    }
    eprintln!("-->{}", warning);
    STATE.lock().unwrap().warning = warning;
    if errors.is_empty() { Ok(()) } else { Err("部分被误回收的文件恢复失败".to_string()) }
}

/// 把隔离区里的文件恢复到原来的位置,并恢复存储记录
async fn restore(record: &StorageRecycle) -> Result<(), String> {
    let err = |e: sqlx::Error| e.to_string();
    let Some(recycle_path) = &record.recyclePath else {
        return Err(format!("存储文件{}移入隔离区时已经不存在,无法恢复", record.path));
    };
    let path = PathBuf::from(&record.path);
    if tokio::fs::try_exists(&path).await.unwrap_or(true) {
        return Err(format!("{}已经存在,无法恢复", record.path));
    }
    let mut ctx = lib_db::get_context();
    ctx.begin_immediate().await.map_err(err)?;
    let storage = StorageFile {
        id: record.id,
        path: record.path.clone(),
        md5: record.md5.clone(),
    };
    storage_file_dao::insert(&mut ctx, &storage).await.map_err(err)?;
    storage_recycle_dao::delete(&mut ctx, record.id).await.map_err(err)?;
    let moved = async {
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        tokio::fs::rename(recycle_path, &path).await
    };
    if let Err(e) = moved.await {
        ctx.rollback().await.map_err(err)?;
        return Err(format!("{}恢复失败:{}", record.path, e));
    }
    if let Err(e) = ctx.commit().await {
        let _ = tokio::fs::rename(&path, recycle_path).await;
        return Err(format!("{}恢复失败:{}", record.path, e));
    }
    Ok(())
}

/// 真正删除隔离区里超过保留天数的文件
async fn purge_recycle() -> Result<(), String> {
    let err = |e: sqlx::Error| e.to_string();
    let time = time_util::current_millis() - recycle_keep_days() * DAY_MILLIS;
    let mut ctx = lib_db::get_context();
    let mut dirs = HashSet::new();
    let mut pre_ids = Vec::new();
    loop {
        let list = storage_recycle_dao::select_before(&mut ctx, time).await.map_err(err)?;
        let ids: Vec<i64> = list.iter().map(|it| it.id).collect();
        if ids.is_empty() || ids == pre_ids {
            break;
        }

        //删除之前再确认一次没有被使用,被使用的留给restore_used恢复
        let used: HashSet<i64> = storage_recycle_dao::select_used(&mut ctx).await.map_err(err)?.into_iter().map(|it| it.id).collect();
        for it in &list {
            if used.contains(&it.id) {
                continue;
            }
            if let Some(recycle_path) = &it.recyclePath {
                match tokio::fs::remove_file(recycle_path).await {
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(format!("删除隔离区文件{}失败:{}", recycle_path, e)),
                }
                if let Some(dir) = Path::new(recycle_path).parent() {
                    dirs.insert(dir.to_path_buf());
                }
            }
            storage_recycle_dao::delete(&mut ctx, it.id).await.map_err(err)?;
        }
        pre_ids = ids;
    }

    //删除空的日期文件夹(不为空时会失败,忽略)
    for dir in dirs {
        let _ = tokio::fs::remove_dir(dir).await;
    }
    Ok(())
}

/// 隔离区统计
pub async fn recycle_stats() -> Result<storage_recycle_dao::StorageRecycleStats, sqlx::Error> {
    storage_recycle_dao::stats(&mut lib_db::get_context()).await
}

/// 回收定时器的状态描述(高级页面显示用)
pub fn state_text() -> String {
    let state = state();
    let mut text = format!("回收定时器{}", if state.is_running { "运行中；" } else { "等待中；" });
    if !state.error.is_empty() {
        text += &format!("错误：{}；", state.error);
    }
    if !state.warning.is_empty() {
        text += &format!("注意：{}；", state.warning);
    }
    if state.last_run_time > 0 {
        text += &format!("上次执行时间：{}", state.last_run_time.date_format());
    }
    text
}
