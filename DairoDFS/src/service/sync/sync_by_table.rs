use crate::dao::sql_log_dao;
use crate::service::sync::sync_handle::{DfsFileAction, DfsFileRow, StorageAction};
use crate::service::sync::sync_info::{SyncInfo, STATE_IDLE, STATE_SYNCING};
use crate::service::sync::table_data::TABLES;
use crate::service::sync::{alias, sync_handle, sync_http, sync_info, table_data, version, IS_TABLE_SYNCING, SYNC_LOCK};
use crate::system_config;
use crate::util::{id_util, time_util};
use serde_json::{Map, Value};
use sqlx_context::DbContext;
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
        crate::service::sync::clear_full_sync_pending();
    }
    crate::service::sync::start_listen(infos);
}

/// 全量同步一台主机(日志同步时发现还没有从这台主机全量同步过时执行)
pub async fn sync_one(info: &SyncInfo) -> Result<(), String> {
    let _lock = SYNC_LOCK.lock().await;
    IS_TABLE_SYNCING.store(true, Ordering::Relaxed);
    let result = sync_by_info(info).await;
    IS_TABLE_SYNCING.store(false, Ordering::Relaxed);
    if result.is_ok() && crate::service::sync::all_synced() {
        crate::service::sync::clear_full_sync_pending();
    }
    result
}

/// 全量同步一台主机的数据
/// 先记下主机当前的日志顺序号,再获取主机的所有数据,之后从这个顺序号开始日志同步
/// 获取数据的过程中主机的数据又有变化时,这些变化在之后的日志同步里再执行一遍(已经存在的不重复添加,修改以最后修改的为准)
async fn sync_by_info(info: &SyncInfo) -> Result<(), String> {
    info.update(|it| {
        it.state = STATE_SYNCING;
        it.msg = "全量同步中".to_string();
    });
    let node = crate::service::sync::request_node_info(&info.url).await?;
    let seq: i64 = sync_http::request(&format!("{}/get_last_seq", info.url))
        .await?
        .trim()
        .parse()
        .map_err(|_| "获取日志顺序号失败".to_string())?;
    let tb_names = TABLES.iter().map(|it| format!("tbNames={}", it)).collect::<Vec<_>>().join("&");
    let count: i64 = sync_http::request(&format!("{}/get_table_count?{}", info.url, tb_names))
        .await?
        .trim()
        .parse()
        .map_err(|_| "获取数据总数失败".to_string())?;
    info.update(|it| {
        it.count = count;
        it.sync_count = 0;
    });
    let mut changed = false;
    for table in TABLES {
        changed |= sync_table(info, table).await?;
    }

    //获取数据的过程中主机又合并了文件夹或存储文件时,前面同步过来的数据可能还引用着被合并掉的ID
    //重新同步一次合并记录,把本机引用了被合并掉的ID的数据都换成保留下来的ID
    changed |= sync_table(info, "sync_alias").await?;
    changed |= apply_alias().await?;

    //还没有执行的这台主机的日志,修改已经包含在全量同步的数据里了
    sql_log_dao::delete_not_run_by_source(&mut lib_db::get_context(), &info.url)
        .await
        .map_err(|e| e.to_string())?;
    crate::service::sync::save_last_seq(&info.url, seq)?;
    crate::service::sync::save_peer_epoch(&info.url, node.epoch)?;
    *info.peer.lock().unwrap() = Some(node);
    if changed {
        //通知级联在本机下面的分机重新全量同步
        crate::service::sync::bump_local_epoch()?;
    }
    info.update(|it| {
        it.state = STATE_IDLE;
        it.msg = "全量同步完成".to_string();
        it.last_time = time_util::current_millis();
    });
    Ok(())
}

/// 同步一张表(按ID顺序分批获取)
/// - return 本机的数据是否有变化
async fn sync_table(info: &SyncInfo, table: &str) -> Result<bool, String> {
    let mut changed = false;
    let mut last_id = i64::MIN;
    loop {
        if info.is_stop() {
            return Err("同步被强制取消".to_string());
        }
        let url = format!("{}/get_table_data?tbName={}&lastId={}", info.url, table, last_id);
        let rows: Vec<Map<String, Value>> =
            serde_json::from_str(&sync_http::request(&url).await?).map_err(|e| format!("表数据格式错误:{}", e))?;
        let Some(last) = rows.last() else {
            return Ok(changed);
        };
        last_id = last.get("id").and_then(|it| it.as_i64()).ok_or("表数据缺少ID")?;
        for row in rows {
            info.update(|it| it.sync_count += 1);
            changed |= insert_row(info, table, row).await?;
        }
    }
}

/// 本机还引用着被合并掉的文件夹、存储文件的数据,换成保留下来的ID
/// - return 本机的数据是否有变化
async fn apply_alias() -> Result<bool, String> {
    let err = |e: sqlx::Error| e.to_string();
    let aliases = table_data::select(&mut lib_db::get_context(), "select id, tb from sync_alias", &[]).await.map_err(err)?;
    let mut changed = false;
    for it in aliases {
        let id = it.get("id").and_then(|it| it.as_i64()).unwrap_or(0);
        let mut ctx = lib_db::get_context();
        ctx.begin_immediate().await.map_err(err)?;
        let target = alias::resolve(&mut ctx, id).await.map_err(err)?;
        let count = match it.get("tb").and_then(|it| it.as_str()) {
            Some("storage_file") => {
                table_data::execute(&mut ctx, "update dfs_file set storageId = ? where storageId = ?", &[target.into(), id.into()]).await.map_err(err)?
                    + table_data::execute(&mut ctx, "update dfs_file_delete set storageId = ? where storageId = ?", &[target.into(), id.into()])
                        .await
                        .map_err(err)?
            }
            Some("dfs_file") => {
                let used = table_data::select(
                    &mut ctx,
                    "select 1 from dfs_file where id = ? or parentId = ? union all select 1 from sub_account_grant where folderId = ? limit 1",
                    &[id.into(), id.into(), id.into()],
                )
                .await
                .map_err(err)?;
                if used.is_empty() {
                    0
                } else {
                    sync_handle::merge_folder(&mut ctx, id, target).await?;
                    1
                }
            }
            _ => 0,
        };
        ctx.commit().await.map_err(err)?;
        changed |= count > 0;
    }
    Ok(changed)
}

/// 同步一行数据,本机已经存在时对比数据是否一致
/// - return 本机的数据是否有变化
async fn insert_row(info: &SyncInfo, table: &str, mut row: Map<String, Value>) -> Result<bool, String> {
    let id = row.get("id").and_then(|it| it.as_i64()).ok_or("表数据缺少ID")?;
    let md5 = row.get("md5").and_then(|it| it.as_str()).unwrap_or_default().to_string();

    //主机端各个字段组的修改时间
    let versions: Vec<(String, i64)> = match row.remove(table_data::VERSION_KEY) {
        Some(Value::Object(it)) => it.into_iter().filter_map(|(k, v)| Some((k, v.as_i64()?))).collect(),
        _ => Vec::new(),
    };
    if let Some(max) = versions.iter().map(|it| it.1).max() {
        id_util::observe(max);
    }

    //需要下载的文件在开启事务之前下载
    let prepared = if table == "storage_file" {
        sync_handle::prepare_storage_file(info, &md5).await?
    } else {
        None
    };
    let err = |e: sqlx::Error| e.to_string();
    let mut ctx = lib_db::get_context();
    ctx.begin_immediate().await.map_err(err)?;
    let int = |row: &Map<String, Value>, key: &str| row.get(key).and_then(|it| it.as_i64()).unwrap_or(0);
    let exists = table_data::select(&mut ctx, &format!("select * from {} where id = ?", table), &[id.into()])
        .await
        .map_err(err)?
        .into_iter()
        .next();
    if exists.is_none() {
        match table {
            "storage_file" => match sync_handle::storage_file(&mut ctx, &md5, id, prepared).await? {
                StorageAction::Insert(path) => {
                    row.insert("path".to_string(), Value::from(path));
                }
                StorageAction::Skip => return ctx.commit().await.map(|_| false).map_err(err),
            },
            "dfs_file" => {
                let (parent_id, storage_id) = sync_handle::remap_dfs_file(&mut ctx, int(&row, "parentId"), int(&row, "storageId")).await?;
                row.insert("parentId".to_string(), Value::from(parent_id));
                row.insert("storageId".to_string(), Value::from(storage_id));
                match sync_handle::dfs_file(&mut ctx, info, &DfsFileRow::from_map(&row)).await? {
                    DfsFileAction::Skip => return ctx.commit().await.map(|_| false).map_err(err),
                    DfsFileAction::InsertAsHistory => {
                        row.insert("isHistory".to_string(), Value::from(1));
                    }
                    DfsFileAction::Insert => {}
                }
            }
            "sub_account_grant" => {
                let folder_id = alias::resolve(&mut ctx, int(&row, "folderId")).await.map_err(err)?;
                row.insert("folderId".to_string(), Value::from(folder_id));
                if !sync_handle::grant(&mut ctx, id, int(&row, "subUserId"), folder_id).await? {
                    return ctx.commit().await.map(|_| false).map_err(err);
                }
            }
            _ => {}
        }
        table_data::insert(&mut ctx, table, &row).await.map_err(err)?;
        save_versions(&mut ctx, table, id, &versions).await.map_err(err)?;
        return ctx.commit().await.map(|_| true).map_err(err);
    }
    let mut exists = exists.unwrap();
    if table == "storage_file" || table == "sync_alias" {
        //存储路径是本机的路径;合并记录在每台服务器上都一样(只有合并时间不同),不需要对比
        return ctx.commit().await.map(|_| false).map_err(err);
    }
    if table == "dfs_file" || table == "sub_account_grant" {
        //两边引用了被合并掉的ID时,都换成保留下来的ID再对比(本机的引用在全量同步的最后统一修改)
        for key in ["parentId", "storageId", "folderId"] {
            for it in [&mut row, &mut exists] {
                if let Some(value) = it.get(key).and_then(|it| it.as_i64()) {
                    if value > 0 {
                        it.insert(key.to_string(), Value::from(alias::resolve(&mut ctx, value).await.map_err(err)?));
                    }
                }
            }
        }
    }

    //按字段组以最后修改的为准合并
    let local_versions = version::select(&mut ctx, table, id).await.map_err(err)?;
    //不一致的字段没有修改时间,无法判断以哪边为准(老版本的数据就不一致)时,停止同步或者使用主机的数据
    let (merged, conflict) = version::merge_row(table, &exists, &row, &versions, &local_versions);
    if let Some(key) = conflict {
        if !system_config::get().ignore_sync_error {
            return Err(format!(
                "表{}数据同步失败，该id[{}]数据已经存在，字段[{}]主机：{}，本机：{}",
                table,
                id,
                key,
                row.get(&key).cloned().unwrap_or_default(),
                exists.get(&key).cloned().unwrap_or_default()
            ));
        }
    }
    let changed = merged != exists;
    if changed {
        table_data::execute(&mut ctx, &format!("delete from {} where id = ?", table), &[id.into()])
            .await
            .map_err(err)?;
        table_data::insert(&mut ctx, table, &merged).await.map_err(err)?;
    }
    save_versions(&mut ctx, table, id, &versions).await.map_err(err)?;
    ctx.commit().await.map(|_| changed).map_err(err)
}

/// 保存主机端各个字段组的修改时间(只会往新的改)
async fn save_versions(ctx: &mut DbContext, table: &str, id: i64, versions: &[(String, i64)]) -> Result<(), sqlx::Error> {
    for (group, ver) in versions {
        version::save(ctx, table, id, group, *ver).await?;
    }
    Ok(())
}
