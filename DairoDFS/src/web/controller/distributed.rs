use crate::dao::sql_log_dao::SqlLog;
use crate::dao::{sql_log_dao, storage_file_dao};
use crate::service::sync::distributed_push::{KEEP_ALIVE_TIME, NOTIFY};
use crate::service::sync::table_data::check_table;
use crate::service::sync::{distributed_push, table_data, IS_LOG_SYNCING, IS_TABLE_SYNCING};
use crate::util::id_util;
use crate::web::biz_error::BizError;
use crate::biz_error;
use bytes::Bytes;
use hyper::HeaderMap;
use hyperoute_core::Stream;
use hyperoute_macro::GET;
use serde_json::{Map, Value};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;

/// 限制每次同步的数据量
const MAX_SYNC_DATA_LIMIT: i64 = 10000;

/// 分机端监听日志的长连接,主机有新的日志时返回1,否则每隔一段时间返回0保持连接
/// - last_id 分机端已经同步到的日志ID
#[GET("/distributed/{token}/{client_token}/listen")]
pub async fn listen(token: String, client_token: String, last_id: i64) -> Stream {
    let _ = token;
    let no = distributed_push::register(&client_token);
    let (tx, rx) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(4);
    tokio::spawn(async move {
        loop {
            //先登记等待,再检查是否有新的日志,避免检查之后、等待之前产生的日志被漏掉
            let notified = NOTIFY.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let max_id = sql_log_dao::select_last_id(&mut lib_db::get_context()).await.ok().flatten().unwrap_or(0);
            if max_id > last_id {
                let _ = tx.send(Ok(Bytes::from_static(&[1]))).await;
                return;
            }
            if tx.send(Ok(Bytes::from_static(&[0]))).await.is_err() {
                return;
            }
            tokio::select! {
                _ = &mut notified => {}
                _ = tokio::time::sleep(Duration::from_secs(KEEP_ALIVE_TIME)) => {}
                _ = tx.closed() => return,
            }
            if !distributed_push::is_current(&client_token, no) {
                //已经被该分机新的连接替代
                return;
            }
        }
    });
    Stream::new(ReceiverStream::new(rx)).header("content-type", "application/octet-stream")
}

/// 获取同步日志
#[GET("/distributed/{token}/{client_token}/get_log")]
pub async fn get_log(last_id: i64) -> Result<Vec<SqlLog>, BizError> {
    Ok(sql_log_dao::select_after_id(&mut lib_db::get_context(), last_id).await?)
}

/// 获取一个断面ID,防止全量同步的过程中主机又增加数据,导致全量同步数据不完整
#[GET("/distributed/{token}/{client_token}/get_aop_id")]
pub async fn get_aop_id() -> String {
    id_util::id().to_string()
}

/// 获取要同步的数据总条数
#[GET("/distributed/{token}/{client_token}/get_table_count")]
pub async fn get_table_count(tb_names: Vec<String>, aop_id: i64) -> Result<String, BizError> {
    let mut ctx = lib_db::get_context();
    let mut count = 0;
    for name in tb_names {
        let table = check_table(&name).map_err(BizError::msg)?;
        let rows = table_data::select(&mut ctx, &format!("select count(*) as c from {} where id < ?", table), &[aop_id.into()]).await?;
        count += rows.first().and_then(|it| it.get("c")).and_then(|it| it.as_i64()).unwrap_or(0);
    }
    Ok(count.to_string())
}

/// 获取表数据
#[GET("/distributed/{token}/{client_token}/get_table_data")]
pub async fn get_table_data(tb_name: String, last_id: i64, aop_id: i64) -> Result<Vec<Map<String, Value>>, BizError> {
    if IS_LOG_SYNCING.load(Ordering::Relaxed) || IS_TABLE_SYNCING.load(Ordering::Relaxed) {
        return Err(biz_error!("主机正在同步数据中，请等待完成后继续。"));
    }
    let table = check_table(&tb_name).map_err(BizError::msg)?;
    let sql = format!("select * from {} where id > ? and id < ? order by id asc limit ?", table);
    Ok(table_data::select(&mut lib_db::get_context(), &sql, &[last_id.into(), aop_id.into(), MAX_SYNC_DATA_LIMIT.into()]).await?)
}

/// 下载文件(支持断点续传)
#[GET("/distributed/{token}/{client_token}/download/{md5}")]
pub async fn download(md5: String, headers: &HeaderMap) -> Result<Stream, BizError> {
    let Some(storage) = storage_file_dao::select_by_md5(&mut lib_db::get_context(), &md5).await? else {
        return Err(BizError::not_found());
    };
    Stream::file(&storage.path, headers).await.map_err(|_| BizError::not_found())
}
