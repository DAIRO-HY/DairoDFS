use crate::dao::sql_log_dao;
use crate::dao::sql_log_dao::SqlLog;
use crate::extension::number::NumberExtension;
use crate::service::sync::sync_handle::{DfsFileAction, DfsFileRow, StorageAction};
use crate::service::sync::sync_http::LISTEN_CLIENT;
use crate::service::sync::sync_info::{SyncInfo, STATE_IDLE, STATE_SYNCING};
use crate::service::sync::version::Decision;
use crate::service::sync::{alias, distributed_push, sync_by_table, sync_handle, sync_http, sync_info, table_data, version, IS_LOG_SYNCING, SYNC_LOCK};
use crate::util::{id_util, time_util};
use serde_json::Value;
use sqlx_context::DbContext;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// 检查主机信息的间隔时间(毫秒)
const CHECK_NODE_INTERVAL: i64 = 60_000;

/// 监听主机的日志变化
pub async fn listen(info: Arc<SyncInfo>) {
    while !info.is_stop() {
        tokio::time::sleep(Duration::from_secs(3)).await;

        //先检查主机的信息(服务器编号、时钟),有问题时不同步
        if time_util::current_millis() - *info.last_check.lock().unwrap() > CHECK_NODE_INTERVAL {
            if let Err(e) = crate::service::sync::check_node(&info).await {
                info.error(e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            *info.last_check.lock().unwrap() = time_util::current_millis();
        }

        //还没有从这台主机全量同步过(新添加的主机、主机地址或票据变了、从老版本升级),先全量同步,不从头执行主机的历史日志
        //主机从其他服务器全量同步过来了新的数据(没有同步日志),也要重新全量同步
        if crate::service::sync::need_full_sync(&info) {
            if let Err(e) = sync_by_table::sync_one(&info).await {
                info.error(format!("全量同步失败:{}", e));
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        }
        listen_once(&info).await;
    }
}

/// 发起一次长连接监听,主机有新的日志时去同步
async fn listen_once(info: &SyncInfo) {

    //监听之前先同步一次,达到上次执行失败,本次重试的目的
    if let Err(e) = sync(info).await {
        info.error(format!("日志同步失败:{}", e));
        return;
    }
    let Some(last_seq) = crate::service::sync::get_last_seq(&info.url) else {
        return;
    };
    let url = format!("{}/listen?lastSeq={}", info.url, last_seq);
    let mut resp = match LISTEN_CLIENT.get(&url).send().await {
        Ok(it) => it,
        Err(e) => {
            info.msg(format!(
                "{}:服务端心跳检查失败，间隔5秒之后会重试。错误：{}",
                time_util::current_millis().date_format(),
                e
            ));
            tokio::time::sleep(Duration::from_secs(5)).await;
            return;
        }
    };
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        info.error(format!("监听日志失败,可能是同步票据无效。Status:{} Body:{}", status, body));
        return;
    }
    loop {
        if info.is_stop() {
            return;
        }
        let chunk = match resp.chunk().await {
            Ok(Some(it)) => it,
            Ok(None) => return,
            Err(e) => {
                info.msg(format!("服务端心跳检查失败。{}", e));
                return;
            }
        };
        for tag in chunk.iter() {
            info.update(|it| it.last_heart_time = time_util::current_millis());
            match tag {
                0 => info.msg("心跳检测中。"),
                1 => {
                    //主机有新的日志
                    if let Err(e) = sync(info).await {
                        info.error(format!("日志同步失败:{}", e));
                    }
                    return;
                }
                _ => {
                    info.msg("监听日志连接返回了非0,1错误，可能是认证token失效。");
                    return;
                }
            }
        }
    }
}

/// 同步主机的日志(单线程执行)
pub async fn sync(info: &SyncInfo) -> Result<(), String> {
    let _lock = SYNC_LOCK.lock().await;
    IS_LOG_SYNCING.store(true, Ordering::Relaxed);
    let result = request_sql_log(info).await;
    IS_LOG_SYNCING.store(false, Ordering::Relaxed);
    result
}

/// 循环获取主机的日志并执行,直到全部同步完成
async fn request_sql_log(info: &SyncInfo) -> Result<(), String> {
    loop {
        if info.is_stop() {
            return Ok(());
        }
        let Some(last_seq) = crate::service::sync::get_last_seq(&info.url) else {
            //还没有全量同步过
            return Ok(());
        };
        let body = sync_http::request(&format!("{}/get_log?lastSeq={}", info.url, last_seq)).await?;
        let logs: Vec<SqlLog> = serde_json::from_str(&body).map_err(|e| format!("日志数据格式错误:{}", e))?;
        if logs.is_empty() {
            run_sql().await?;
            info.update(|it| {
                it.state = STATE_IDLE;
                it.msg = String::new();
                it.last_time = time_util::current_millis();
            });
            return Ok(());
        }
        info.update(|it| {
            it.state = STATE_SYNCING;
            it.msg = "日志同步中".to_string();
            it.count += logs.len() as i64;
        });

        //先把日志保存到本机,再逐条执行
        let mut ctx = lib_db::get_context();
        for it in &logs {
            if sql_log_dao::select_one(&mut ctx, it.id).await.is_ok() {
                //这条日志已经添加(从其他主机同步过来过,或者是本机自己的日志)
                continue;
            }
            let param = sql_log_dao::InsertWithSeqParam {
                id: it.id,
                log_time: it.date,
                sql: it.sql.clone(),
                param: it.param.clone(),
                state: 0,
                source: info.url.clone(),
                err: None,
            };
            sql_log_dao::insert_with_seq(&mut ctx, param).await.map_err(|e| e.to_string())?;
        }

        //保存成功之后立即记录最后的顺序号,避免重复获取
        crate::service::sync::save_last_seq(&info.url, logs.last().unwrap().seq)?;

        //通知级联在本机下面的分机
        distributed_push::push();
        run_sql().await?;
    }
}

/// 执行本机还没有执行的同步日志
async fn run_sql() -> Result<(), String> {
    loop {
        let list = sql_log_dao::select_not_run(&mut lib_db::get_context()).await.map_err(|e| e.to_string())?;
        if list.is_empty() {
            return Ok(());
        }
        for log in list {
            let Some(info) = sync_info::find(&log.source) else {
                let err = "该日志的来源主机已经不在同步列表中".to_string();
                set_error(log.id, &err).await;
                return Err(err);
            };
            if let Err(e) = apply(&info, &log).await {
                set_error(log.id, &e).await;
                return Err(e);
            }
            info.update(|it| it.sync_count += 1);
        }
    }
}

/// 记录日志执行失败
async fn set_error(id: i64, err: &str) {
    let _ = sql_log_dao::set_state(&mut lib_db::get_context(), id, 2, Some(err.to_string())).await;
}

/// 添加语句的表名
fn insert_table(normalized: &str) -> Option<&str> {
    let rest = normalized.strip_prefix("insertinto")?;
    rest.find('(').map(|it| &rest[..it])
}

/// 记录是否已经存在
async fn exists(ctx: &mut DbContext, table: &str, id: Option<i64>) -> Result<bool, String> {
    let Some(id) = id else {
        return Ok(false);
    };
    let rows = table_data::select(ctx, &format!("select 1 from {} where id = ?", table), &[id.into()])
        .await
        .map_err(|e| e.to_string())?;
    Ok(!rows.is_empty())
}

/// 第i个参数的整数值
fn int(params: &[Value], i: usize) -> i64 {
    params.get(i).and_then(|it| it.as_i64()).unwrap_or(0)
}

/// 执行一条同步日志,执行SQL和标记执行完成在同一个事务里
/// 多台服务器双向同步时,同一条日志可能从多台主机同步过来,执行的顺序也可能不一样,所以:
/// - 添加的记录已经存在时不再添加
/// - 引用了被合并掉的文件夹、存储文件时,换成保留下来的ID
/// - 修改时本机已经有更新的修改,以最后修改的为准
async fn apply(info: &SyncInfo, log: &SqlLog) -> Result<(), String> {
    let mut params: Vec<Value> = serde_json::from_str(&log.param).map_err(|e| format!("日志参数格式错误:{}", e))?;

    //之后本机的修改一定排在这条修改的后面
    id_util::observe(log.id);

    //用来判断是哪一种SQL语句
    let normalized = version::normalize(&log.sql);
    let table = insert_table(&normalized).map(|it| it.to_string());

    //需要下载的文件在开启事务之前下载
    let prepared = if table.as_deref() == Some("storage_file") {
        let md5 = params.get(2).and_then(|it| it.as_str()).ok_or("存储文件的MD5参数不正确")?.to_string();
        sync_handle::prepare_storage_file(info, &md5).await?
    } else {
        None
    };
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    ctx.begin_immediate().await.map_err(err)?;
    let mut skip = false;
    let mut versioned = None;
    match table.as_deref() {
        Some("storage_file") => {
            let md5 = params[2].as_str().unwrap_or_default().to_string();
            match sync_handle::storage_file(&mut ctx, &md5, int(&params, 0), prepared).await? {
                StorageAction::Insert(path) => params[1] = Value::from(path),
                StorageAction::Skip => skip = true,
            }
        }
        Some("dfs_file") => {
            let (parent_id, storage_id) = sync_handle::remap_dfs_file(&mut ctx, int(&params, 2), int(&params, 7)).await?;
            params[2] = Value::from(parent_id);
            params[7] = Value::from(storage_id);
            if exists(&mut ctx, "dfs_file", Some(int(&params, 0))).await? {
                skip = true;
            } else {
                let row = DfsFileRow::from_params(&params).ok_or("用户文件的参数不正确")?;
                match sync_handle::dfs_file(&mut ctx, info, &row).await? {
                    DfsFileAction::Skip => skip = true,
                    //先设置为历史版本再添加,否则会和本机的同名文件冲突(参数顺序与dfs_file表的列顺序一致)
                    DfsFileAction::InsertAsHistory => params[11] = Value::from(1),
                    DfsFileAction::Insert => {}
                }
            }
        }
        Some("dfs_file_delete") => {
            //从用户文件复制到彻底删除表的语句,第二个参数是ID
            skip = exists(&mut ctx, "dfs_file_delete", Some(int(&params, 1))).await?;
        }
        Some("sub_account_grant") => {
            let folder_id = alias::resolve(&mut ctx, int(&params, 2)).await.map_err(err)?;
            params[2] = Value::from(folder_id);
            let id = int(&params, 0);
            skip = exists(&mut ctx, "sub_account_grant", Some(id)).await?
                || !sync_handle::grant(&mut ctx, id, int(&params, 1), folder_id).await?;
        }
        Some(table) if table_data::check_table(table).is_ok() => {
            skip = exists(&mut ctx, table, Some(int(&params, 0))).await?;
        }
        Some(_) => {}
        None => {
            if let Some(stmt) = version::find(&log.sql) {
                let is_move = stmt.table == "dfs_file" && stmt.columns.iter().any(|it| it.0 == "parentId");
                if is_move {
                    //移动到被合并掉的文件夹时,换成保留下来的文件夹
                    params[0] = Value::from(alias::resolve(&mut ctx, int(&params, 0)).await.map_err(err)?);
                }
                match version::decide(&mut ctx, stmt, params.clone(), log.id).await.map_err(err)? {
                    Decision::Skip => skip = true,
                    Decision::Execute { params: new_params, groups } => {
                        params = new_params;
                        if is_move && groups.contains(&"pos") {
                            //两台服务器同时把不同的文件移动或重命名成同一个名字
                            let name = params[1].as_str().unwrap_or_default().to_string();
                            if !sync_handle::move_conflict(&mut ctx, info, int(&params, 2), int(&params, 0), &name).await? {
                                skip = true;
                            }
                        }
                        versioned = Some((stmt, groups));
                    }
                }
            }
        }
    }
    if !skip {
        table_data::execute(&mut ctx, &log.sql, &params).await.map_err(err)?;
        if let Some((stmt, groups)) = versioned {
            version::save_groups(&mut ctx, stmt, &params, &groups, log.id).await.map_err(err)?;
        }
    }
    sql_log_dao::set_state(&mut ctx, log.id, 1, None).await.map_err(err)?;
    ctx.commit().await.map_err(err)
}
