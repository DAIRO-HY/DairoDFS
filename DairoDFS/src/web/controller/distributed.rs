use crate::dao::sql_log_dao::SqlLog;
use crate::dao::{sql_log_dao, storage_file_dao};
use crate::service::sync::distributed_push::{KEEP_ALIVE_TIME, NOTIFY};
use crate::service::sync::table_data::check_table;
use crate::service::sync::{distributed_push, table_data, version, NodeInfo};
use crate::system_config;
use crate::util::time_util;
use crate::web::biz_error::BizError;
use bytes::Bytes;
use hyper::HeaderMap;
use hyperoute_core::Stream;
use hyperoute_macro::GET;
use serde_json::{Map, Value};
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;

/// 限制每次同步的数据量
const MAX_SYNC_DATA_LIMIT: i64 = 10000;

/// 分机端监听日志的长连接,主机有新的日志时返回1,否则每隔一段时间返回0保持连接
/// - last_seq 分机端已经同步到的日志顺序号
#[GET("/distributed/{token}/{client_token}/listen")]
pub async fn listen(token: String, client_token: String, last_seq: i64) -> Stream {
    let _ = token;
    let no = distributed_push::register(&client_token);
    let (tx, rx) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(4);
    tokio::spawn(async move {
        loop {
            //先登记等待,再检查是否有新的日志,避免检查之后、等待之前产生的日志被漏掉
            let notified = NOTIFY.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let max_seq = sql_log_dao::select_last_seq(&mut lib_db::get_context()).await.ok().flatten().unwrap_or(0);
            if max_seq > last_seq {
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

/// 获取同步日志(按添加到本机的顺序)
#[GET("/distributed/{token}/{client_token}/get_log")]
pub async fn get_log(last_seq: i64) -> Result<Vec<SqlLog>, BizError> {
    Ok(sql_log_dao::select_after_seq(&mut lib_db::get_context(), last_seq).await?)
}

/// 获取全量同步的断面:本机已经执行完的日志顺序号,全量同步完成之后分机从这里开始日志同步
/// 本机收到了但是还没有执行的日志,修改还不在数据里,让分机自己再执行一遍(已经执行过的修改再执行一遍结果不变)
#[GET("/distributed/{token}/{client_token}/get_last_seq")]
pub async fn get_last_seq() -> Result<String, BizError> {
    let rows = table_data::select(
        &mut lib_db::get_context(),
        "select coalesce((select min(seq) - 1 from sql_log where state != 1), (select max(seq) from sql_log), 0) as seq",
        &[],
    )
    .await?;
    Ok(rows.first().and_then(|it| it.get("seq")).and_then(|it| it.as_i64()).unwrap_or(0).to_string())
}

/// 获取本机的信息(集群ID、服务器编号、是否只读、当前时间),分机用来检查服务器编号和时钟
#[GET("/distributed/{token}/{client_token}/get_node_info")]
pub async fn get_node_info() -> NodeInfo {
    let config = system_config::get();
    NodeInfo {
        cluster_id: config.cluster_id.clone(),
        node_no: config.node_no,
        is_read_only: config.is_read_only,
        time: time_util::current_millis(),
        epoch: crate::service::sync::local_epoch(),
    }
}

/// 获取集群ID,分机使用第一台同步主机的集群ID
#[GET("/distributed/{token}/{client_token}/get_cluster_id")]
pub async fn get_cluster_id() -> String {
    system_config::get().cluster_id.clone()
}

/// 获取要同步的数据总条数
#[GET("/distributed/{token}/{client_token}/get_table_count")]
pub async fn get_table_count(tb_names: Vec<String>) -> Result<String, BizError> {
    let mut ctx = lib_db::get_context();
    let mut count = 0;
    for name in tb_names {
        let table = check_table(&name).map_err(BizError::msg)?;
        let rows = table_data::select(&mut ctx, &format!("select count(*) as c from {}", table), &[]).await?;
        count += rows.first().and_then(|it| it.get("c")).and_then(|it| it.as_i64()).unwrap_or(0);
    }
    Ok(count.to_string())
}

/// 获取表数据(按ID顺序分批获取),需要按最后修改为准的表,每行附带各个字段组的修改时间
#[GET("/distributed/{token}/{client_token}/get_table_data")]
pub async fn get_table_data(tb_name: String, last_id: i64) -> Result<Vec<Map<String, Value>>, BizError> {
    let table = check_table(&tb_name).map_err(BizError::msg)?;
    let mut ctx = lib_db::get_context();
    let sql = format!("select * from {} where id > ? order by id asc limit ?", table);
    let mut rows = table_data::select(&mut ctx, &sql, &[last_id.into(), MAX_SYNC_DATA_LIMIT.into()]).await?;
    if !version::is_versioned(table) || rows.is_empty() {
        return Ok(rows);
    }
    let first = rows.first().and_then(|it| it.get("id")).cloned().unwrap_or_default();
    let last = rows.last().and_then(|it| it.get("id")).cloned().unwrap_or_default();
    let versions = table_data::select(
        &mut ctx,
        "select rowId, grp, ver from sync_version where tb = ? and rowId >= ? and rowId <= ?",
        &[table.into(), first, last],
    )
    .await?;
    let mut map: std::collections::HashMap<i64, Map<String, Value>> = std::collections::HashMap::new();
    for it in versions {
        let (Some(row_id), Some(group), Some(ver)) = (
            it.get("rowId").and_then(|it| it.as_i64()),
            it.get("grp").and_then(|it| it.as_str()),
            it.get("ver").cloned(),
        ) else {
            continue;
        };
        map.entry(row_id).or_default().insert(group.to_string(), ver);
    }
    for row in rows.iter_mut() {
        let id = row.get("id").and_then(|it| it.as_i64()).unwrap_or(0);
        if let Some(versions) = map.remove(&id) {
            row.insert(table_data::VERSION_KEY.to_string(), Value::Object(versions));
        }
    }
    Ok(rows)
}

/// 下载文件(支持断点续传)
#[GET("/distributed/{token}/{client_token}/download/{md5}")]
pub async fn download(md5: String, headers: &HeaderMap) -> Result<Stream, BizError> {
    let Some(storage) = storage_file_dao::select_by_md5(&mut lib_db::get_context(), &md5).await? else {
        return Err(BizError::not_found());
    };
    Stream::file_with_etag(&storage.path, headers, &storage.md5).await.map_err(|_| BizError::not_found())
}
