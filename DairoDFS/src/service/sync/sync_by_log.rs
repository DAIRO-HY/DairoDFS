use crate::dao::sql_log_dao;
use crate::dao::sql_log_dao::SqlLog;
use crate::extension::number::NumberExtension;
use crate::service::sync::sync_handle::{DfsFileAction, DfsFileRow};
use crate::service::sync::sync_http::LISTEN_CLIENT;
use crate::service::sync::sync_info::{SyncInfo, STATE_IDLE, STATE_SYNCING};
use crate::service::sync::{distributed_push, sync_handle, sync_http, sync_info, table_data, IS_LOG_SYNCING, SYNC_LOCK};
use crate::util::time_util;
use serde_json::Value;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// 监听主机的日志变化
pub async fn listen(info: Arc<SyncInfo>) {
    while !info.is_stop() {
        tokio::time::sleep(Duration::from_secs(3)).await;
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
    let url = format!("{}/listen?lastId={}", info.url, crate::service::sync::get_last_id(&info.url));
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
        let last_id = crate::service::sync::get_last_id(&info.url);
        let body = sync_http::request(&format!("{}/get_log?lastId={}", info.url, last_id)).await?;
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
                //这条日志已经添加
                continue;
            }
            let log = SqlLog {
                state: 0,
                source: info.url.clone(),
                err: None,
                ..it.clone()
            };
            sql_log_dao::insert(&mut ctx, &log).await.map_err(|e| e.to_string())?;
        }

        //保存成功之后立即记录最后的ID,避免重复获取
        crate::service::sync::save_last_id(&info.url, logs.last().unwrap().id)?;

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

/// 执行一条同步日志,执行SQL和标记执行完成在同一个事务里
async fn apply(info: &SyncInfo, log: &SqlLog) -> Result<(), String> {
    let mut params: Vec<Value> = serde_json::from_str(&log.param).map_err(|e| format!("日志参数格式错误:{}", e))?;

    //用来判断是哪一种SQL语句
    let normalized: String = log.sql.chars().filter(|it| !it.is_whitespace()).collect::<String>().to_lowercase();
    let is_storage_file = normalized.starts_with("insertintostorage_file(");
    let is_dfs_file = normalized.starts_with("insertintodfs_file(");

    //需要下载的文件在开启事务之前下载
    let prepared = if is_storage_file {
        let md5 = params.get(2).and_then(|it| it.as_str()).ok_or("存储文件的MD5参数不正确")?.to_string();
        sync_handle::prepare_storage_file(info, &md5).await?
    } else {
        None
    };
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    ctx.begin().await.map_err(err)?;
    let mut skip = false;
    let mut history_id = None;
    if is_storage_file {
        let id = params[0].as_i64().ok_or("存储文件的ID参数不正确")?;
        let md5 = params[2].as_str().unwrap_or_default().to_string();
        if crate::dao::storage_file_dao::select_one(&mut ctx, id).await.is_ok() {
            //该存储文件已经存在(全量同步时已经同步过)
            skip = true;
        } else {
            params[1] = Value::from(sync_handle::storage_file(&mut ctx, &md5, id, prepared).await?);
        }
    } else if is_dfs_file {
        let row = DfsFileRow::from_params(&params).ok_or("用户文件的参数不正确")?;
        match sync_handle::dfs_file(&mut ctx, info, &row).await? {
            DfsFileAction::Skip => skip = true,
            DfsFileAction::InsertAsHistory => history_id = Some(row.id),
            DfsFileAction::Insert => {}
        }
    }
    if !skip {
        table_data::execute(&mut ctx, &log.sql, &params).await.map_err(err)?;
    }
    if let Some(id) = history_id {
        table_data::execute(&mut ctx, "update dfs_file set isHistory = 1 where id = ?", &[id.into()]).await.map_err(err)?;
    }
    sql_log_dao::set_state(&mut ctx, log.id, 1, None).await.map_err(err)?;
    ctx.commit().await.map_err(err)
}
