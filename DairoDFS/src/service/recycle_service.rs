use crate::dao::{dfs_file_dao, dfs_file_delete_dao, sql_log_dao, storage_file_dao};
use crate::extension::number::NumberExtension;
use crate::service::dfs_file_delete_service;
use crate::util::{db_backup_util, time_util};
use crate::web::biz_error::BizError;
use crate::{db, system_config};
use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use std::sync::Mutex;
use std::time::Duration;

/// 一天的毫秒数
const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// 同步日志保留天数
const SQL_LOG_KEEP_DAYS: i64 = 100;

/// 回收定时器的状态
static STATE: Mutex<TimerState> = Mutex::new(TimerState {
    is_running: false,
    error: String::new(),
    last_run_time: 0,
});

/// 回收定时器的状态
#[derive(Debug, Clone)]
pub struct TimerState {

    // 是否正在运行
    pub is_running: bool,

    // 上次执行的错误信息
    pub error: String,

    // 上次执行完成的时间
    pub last_run_time: i64,
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
    {
        let mut state = STATE.lock().unwrap();
        if state.is_running {
            return;
        }
        state.is_running = true;
    }
    let result = run_inner().await;
    let mut state = STATE.lock().unwrap();
    state.is_running = false;
    state.error = result.err().unwrap_or_default();
    state.last_run_time = time_util::current_millis();
}

async fn run_inner() -> Result<(), String> {
    db_backup_util::backup().await?;
    delete_sql_log().await?;
    delete_trash_timeout().await.map_err(|e| e.to_string())?;
    recycle_deleted_file().await?;
    let delete_time = time_util::current_millis() - system_config::get().delete_storage_timeout * DAY_MILLIS;
    recycle_not_use_storage(delete_time).await
}

/// 删除超出指定时间的同步日志
async fn delete_sql_log() -> Result<(), String> {
    let time = time_util::current_millis() - SQL_LOG_KEEP_DAYS * DAY_MILLIS;
    sql_log_dao::delete_before(&mut lib_db::get_context(), time).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 彻底删除垃圾箱里到期的文件
/// 这一步会记录同步日志,分机跟着主机同步,所以只读模式下不执行
async fn delete_trash_timeout() -> Result<(), BizError> {
    if system_config::get().is_read_only {
        return Ok(());
    }
    let mut pre_ids = Vec::new();
    loop {
        let time = time_util::current_millis() - system_config::get().trash_timeout * DAY_MILLIS;
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

/// 回收已经彻底删除的文件
/// 回收存储文件不记录同步日志,每台机器各自回收,防止文件误删
async fn recycle_deleted_file() -> Result<(), String> {
    let mut pre_ids = Vec::new();
    loop {
        let time = time_util::current_millis() - system_config::get().delete_storage_timeout * DAY_MILLIS;
        let mut ctx = lib_db::get_context();
        let list = dfs_file_delete_dao::select_by_timeout(&mut ctx, time).await.map_err(|e| e.to_string())?;
        let ids: Vec<i64> = list.iter().map(|it| it.id).collect();
        if ids.is_empty() || ids == pre_ids {
            return Ok(());
        }
        let mut storage_ids: Vec<i64> = list.iter().map(|it| it.storageId).filter(|it| *it > 0).collect();
        storage_ids.sort();
        storage_ids.dedup();
        for id in &ids {
            dfs_file_delete_dao::delete(&mut ctx, *id).await.map_err(|e| e.to_string())?;
        }
        for id in storage_ids {
            delete_not_use_storage(id).await?;
        }
        pre_ids = ids;
    }
}

/// 回收没有被使用的存储文件
/// - time 只回收这个时间之前创建的存储文件,避免正在上传的文件被回收
pub async fn recycle_not_use_storage(time: i64) -> Result<(), String> {
    let mut pre_ids = Vec::new();
    loop {
        let ids = storage_file_dao::select_not_use_ids(&mut lib_db::get_context(), time).await.map_err(|e| e.to_string())?;
        if ids.is_empty() || ids == pre_ids {
            return Ok(());
        }
        for id in &ids {
            delete_not_use_storage(*id).await?;
        }
        pre_ids = ids;
    }
}

/// 删除没有被使用的存储文件(删除磁盘文件失败时保留记录)
async fn delete_not_use_storage(id: i64) -> Result<(), String> {
    let mut ctx = lib_db::get_context();
    let err = |e: sqlx::Error| e.to_string();
    if dfs_file_dao::count_by_storage_id(&mut ctx, id).await.map_err(err)? > 0 {
        return Ok(());
    }
    if dfs_file_delete_dao::count_by_storage_id(&mut ctx, id).await.map_err(err)? > 0 {
        return Ok(());
    }
    let Ok(storage) = storage_file_dao::select_one(&mut ctx, id).await else {
        return Ok(());
    };
    match tokio::fs::remove_file(&storage.path).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("文件{}删除失败:{}", storage.path, e)),
    }
    storage_file_dao::delete(&mut ctx, id).await.map_err(err)?;
    Ok(())
}

/// 回收定时器的状态描述(高级页面显示用)
pub fn state_text() -> String {
    let state = state();
    let mut text = format!("回收定时器{}", if state.is_running { "运行中；" } else { "等待中；" });
    if !state.error.is_empty() {
        text += &format!("错误：{}；", state.error);
    }
    if state.last_run_time > 0 {
        text += &format!("上次执行时间：{}", state.last_run_time.date_format());
    }
    text
}
