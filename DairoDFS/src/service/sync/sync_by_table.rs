use crate::dao::sql_log_dao;
use crate::service::sync::sync_handle::{DfsFileAction, DfsFileRow};
use crate::service::sync::sync_info::{SyncInfo, STATE_IDLE, STATE_SYNCING};
use crate::service::sync::table_data::TABLES;
use crate::service::sync::{sync_handle, sync_http, sync_info, table_data, IS_TABLE_SYNCING, SYNC_LOCK};
use crate::system_config;
use crate::util::time_util;
use serde_json::{Map, Value};
use std::sync::atomic::Ordering;

/// 全量同步:从主机下载所有数据和本机数据对比
/// 避免由于网络延迟,本机没有完全同步主机日志(主要是update,delete语句),导致本机数据与主机不一致
pub async fn sync_all() {
    crate::service::sync::stop_all();
    let infos = sync_info::reload();
    {
        let _lock = SYNC_LOCK.lock().await;
        IS_TABLE_SYNCING.store(true, Ordering::Relaxed);
        for info in &infos {
            if let Err(e) = sync_by_info(info).await {
                info.error(format!("全量同步失败:{}", e));
            }
        }
        IS_TABLE_SYNCING.store(false, Ordering::Relaxed);
    }
    if !sync_info::has_error() {
        //全量同步完成,开始日志同步
        crate::service::sync::start_listen(infos);
    }
}

/// 全量同步一台主机的数据
async fn sync_by_info(info: &SyncInfo) -> Result<(), String> {
    info.update(|it| {
        it.state = STATE_SYNCING;
        it.msg = "全量同步中".to_string();
    });

    //从主机获取断面ID,避免同步过程中主机数据发生变化导致数据不一致
    let aop_id: i64 = sync_http::request(&format!("{}/get_aop_id", info.url))
        .await?
        .trim()
        .parse()
        .map_err(|_| "获取断面ID失败".to_string())?;
    let tb_names = TABLES.iter().map(|it| format!("tbNames={}", it)).collect::<Vec<_>>().join("&");
    let count: i64 = sync_http::request(&format!("{}/get_table_count?aopId={}&{}", info.url, aop_id, tb_names))
        .await?
        .trim()
        .parse()
        .map_err(|_| "获取数据总数失败".to_string())?;
    info.update(|it| {
        it.count = count;
        it.sync_count = 0;
    });
    for table in TABLES {
        let mut last_id = 0;
        loop {
            if info.is_stop() {
                return Err("同步被强制取消".to_string());
            }
            let url = format!("{}/get_table_data?tbName={}&lastId={}&aopId={}", info.url, table, last_id, aop_id);
            let rows: Vec<Map<String, Value>> =
                serde_json::from_str(&sync_http::request(&url).await?).map_err(|e| format!("表数据格式错误:{}", e))?;
            let Some(last) = rows.last() else {
                break;
            };
            last_id = last.get("id").and_then(|it| it.as_i64()).ok_or("表数据缺少ID")?;
            for row in rows {
                info.update(|it| it.sync_count += 1);
                insert_row(info, table, row).await?;
            }
        }
    }

    //删除已经同步完成的主机日志,之后从断面ID开始日志同步
    sql_log_dao::delete_by_source(&mut lib_db::get_context(), &info.url, aop_id)
        .await
        .map_err(|e| e.to_string())?;
    crate::service::sync::save_last_id(&info.url, aop_id)?;
    info.update(|it| {
        it.state = STATE_IDLE;
        it.msg = "全量同步完成".to_string();
        it.last_time = time_util::current_millis();
    });
    Ok(())
}

/// 同步一行数据,本机已经存在时对比数据是否一致
async fn insert_row(info: &SyncInfo, table: &str, mut row: Map<String, Value>) -> Result<(), String> {
    let id = row.get("id").and_then(|it| it.as_i64()).ok_or("表数据缺少ID")?;
    let md5 = row.get("md5").and_then(|it| it.as_str()).unwrap_or_default().to_string();

    //需要下载的文件在开启事务之前下载
    let prepared = if table == "storage_file" {
        sync_handle::prepare_storage_file(info, &md5).await?
    } else {
        None
    };
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    ctx.begin().await.map_err(err)?;
    if table == "storage_file" {
        row.insert("path".to_string(), Value::from(sync_handle::storage_file(&mut ctx, &md5, id, prepared).await?));
    } else if table == "dfs_file" {
        if sync_handle::dfs_file(&mut ctx, info, &DfsFileRow::from_map(&row)).await? == DfsFileAction::InsertAsHistory {
            row.insert("isHistory".to_string(), Value::from(1));
        }
    }
    let exists = table_data::select(&mut ctx, &format!("select * from {} where id = ?", table), &[id.into()])
        .await
        .map_err(err)?;
    if let Some(exists) = exists.first() {
        if *exists == row {
            //数据没有变化
            return ctx.commit().await.map_err(err);
        }
        if !system_config::get().ignore_sync_error {
            let key = row.keys().find(|key| exists.get(*key) != row.get(*key)).cloned().unwrap_or_default();
            return Err(format!(
                "表{}数据同步失败，该id[{}]数据已经存在，字段[{}]主机：{}，本机：{}",
                table,
                id,
                key,
                row.get(&key).cloned().unwrap_or_default(),
                exists.get(&key).cloned().unwrap_or_default()
            ));
        }

        //忽略同步错误时,删掉本机的数据,使用主机的数据
        table_data::execute(&mut ctx, &format!("delete from {} where id = ?", table), &[id.into()])
            .await
            .map_err(err)?;
    }
    table_data::insert(&mut ctx, table, &row).await.map_err(err)?;
    ctx.commit().await.map_err(err)
}
