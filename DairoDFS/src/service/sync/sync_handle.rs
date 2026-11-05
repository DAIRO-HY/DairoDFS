use crate::application;
use crate::dao::{dfs_file_dao, storage_file_dao, user_dao};
use crate::extension::number::NumberExtension;
use crate::service::sync::sync_http::DOWNLOAD_CLIENT;
use crate::service::sync::sync_info::SyncInfo;
use crate::service::sync::{alias, table_data};
use crate::service::{dfs_file_service, storage_service};
use crate::util::file_util;
use serde_json::Value;
use sqlx_context::DbContext;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// 下载失败时的重试次数
const RETRY_TIMES: usize = 5;

/// 添加用户文件之前的处理结果
#[derive(Debug, PartialEq)]
pub enum DfsFileAction {

    // 继续添加
    Insert,

    // 该文件已经存在(同一个ID),不需要再添加
    Skip,

    // 本机的同名文件比较新,主机端的文件添加之后设置为历史版本
    InsertAsHistory,
}

/// 同步存储文件之前,本机不存在该文件时先从主机下载
/// 下载可能很久,所以要在开启事务之前执行,避免长时间占用数据库
/// - return 下载之后的存储路径,本机已经存在该文件时返回None
pub async fn prepare_storage_file(info: &SyncInfo, md5: &str) -> Result<Option<String>, String> {
    let exists = storage_file_dao::select_by_md5(&mut lib_db::get_context(), md5).await.map_err(|e| e.to_string())?;
    if exists.is_some() {
        return Ok(None);
    }
    let temp_path = download(info, md5).await?;
    let temp_md5 = file_util::md5(&temp_path).await.map_err(|e| e.to_string())?;
    if temp_md5 != md5 {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!("同步的文件数据不完整，目标文件MD5:{}，实际文件MD5:{}", md5, temp_md5));
    }
    let path = storage_service::move_to_storage(&temp_path).await.map_err(|e| e.to_string())?;
    Ok(Some(path.to_string_lossy().to_string()))
}

/// 同步存储文件的处理结果
#[derive(Debug, PartialEq)]
pub enum StorageAction {

    // 添加,存储路径为本机的路径
    Insert(String),

    // 不需要添加(已经存在,或者本机有同样内容的文件并且ID比较小)
    Skip,
}

/// 同步存储文件(在事务中执行)
/// 本机已经有同样内容(MD5)的文件时,不重复保存,保留ID小的记录,另一个ID记录为被合并
/// 每台服务器都按同样的规则处理,合并的结果一样
/// - prepared prepare_storage_file下载的文件路径
pub async fn storage_file(ctx: &mut DbContext, md5: &str, master_id: i64, prepared: Option<String>) -> Result<StorageAction, String> {
    let err = |e: sqlx::Error| e.to_string();
    if storage_file_dao::select_one(&mut *ctx, master_id).await.is_ok() {
        //该存储文件已经存在
        return Ok(StorageAction::Skip);
    }
    let exists = storage_file_dao::select_by_md5(&mut *ctx, md5).await.map_err(err)?;
    match exists {
        //本机的ID比较大,换成主机的ID,本机的ID记录为被合并
        Some(it) if master_id < it.id => {
            table_data::execute(ctx, "delete from storage_file where id = ?", &[it.id.into()]).await.map_err(err)?;
            table_data::execute(ctx, "update dfs_file set storageId = ? where storageId = ?", &[master_id.into(), it.id.into()]).await.map_err(err)?;
            table_data::execute(ctx, "update dfs_file_delete set storageId = ? where storageId = ?", &[master_id.into(), it.id.into()]).await.map_err(err)?;
            alias::add(ctx, it.id, master_id, "storage_file").await.map_err(err)?;
            Ok(StorageAction::Insert(it.path))
        }

        //本机的ID比较小,保留本机的,主机的ID记录为被合并
        Some(it) => {
            alias::add(ctx, master_id, it.id, "storage_file").await.map_err(err)?;
            Ok(StorageAction::Skip)
        }
        None => prepared.map(StorageAction::Insert).ok_or_else(|| format!("文件{}没有下载", md5)),
    }
}

/// 从主机下载文件,支持断点续传,出错时自动重试
/// - return 下载完成的临时文件
async fn download(info: &SyncInfo, md5: &str) -> Result<PathBuf, String> {
    //不能和网页断点续传的临时文件(temp/md5)同名
    let temp_path = application::TEMP_PATH.join(format!("sync-{}", md5));
    let url = format!("{}/download/{}", info.url, md5);
    let mut retry = 0;
    loop {
        if info.is_stop() {
            return Err("同步被强制取消".to_string());
        }
        match download_once(info, &url, &temp_path).await {
            Ok(_) => return Ok(temp_path),
            Err(e) if retry < RETRY_TIMES => {
                retry += 1;
                info.msg(format!("文件下载失败(正在第{}次尝试，3秒后重试)：{}", retry, e));
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

/// 下载一次,从已经下载的位置继续下载
async fn download_once(info: &SyncInfo, url: &str, temp_path: &PathBuf) -> Result<(), String> {
    let start = tokio::fs::metadata(temp_path).await.map(|it| it.len()).unwrap_or(0);
    let mut resp = DOWNLOAD_CLIENT
        .get(url)
        .header("range", format!("bytes={}-", start))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if status == 416 {
        //文件已经下载完成;空文件请求bytes=0-时也返回416,这时临时文件还没有创建
        if start == 0 {
            tokio::fs::write(temp_path, b"").await.map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if status != 200 && status != 206 {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Status:{}  Body:{}", status, body));
    }

    //主机不支持断点续传时,从头开始下载
    let start = if status == 200 { 0 } else { start };
    let total = resp.content_length().unwrap_or(0) + start;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(status == 206)
        .truncate(status == 200)
        .open(temp_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded = start;
    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        if info.is_stop() {
            return Err("同步被强制取消".to_string());
        }
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        info.msg(format!("正在同步文件：{}/{}", downloaded.data_size(), total.data_size()));
    }
    file.flush().await.map_err(|e| e.to_string())?;
    if downloaded != total {
        return Err(format!("文件下载不完整,Content-Length:{} 实际下载:{}", total, downloaded));
    }
    Ok(())
}

/// 同步过来的用户文件里引用了被合并掉的文件夹、存储文件时,换成保留下来的ID
/// - return (父文件夹ID, 存储文件ID)
pub async fn remap_dfs_file(ctx: &mut DbContext, parent_id: i64, storage_id: i64) -> Result<(i64, i64), String> {
    let err = |e: sqlx::Error| e.to_string();
    let parent_id = alias::resolve(ctx, parent_id).await.map_err(err)?;
    let storage_id = if storage_id > 0 {
        alias::resolve(ctx, storage_id).await.map_err(err)?
    } else {
        storage_id
    };
    Ok((parent_id, storage_id))
}

/// 添加用户文件之前,处理本机已经存在同名文件的情况
/// 每台服务器都按同样的规则处理,结果一样:
/// - 都是文件夹: 保留ID小的(先创建的),另一个文件夹下的内容移过来,它的ID记录为被合并
/// - 都是附属文件: 保留ID小的
/// - 都是文件: ID大的(比较新的)作为当前版本,另一个设置为历史版本
/// - 一个是文件一个是文件夹: 无法同步,需要人工处理
pub async fn dfs_file(ctx: &mut DbContext, info: &SyncInfo, row: &DfsFileRow) -> Result<DfsFileAction, String> {
    if row.is_history || row.is_deleted {
        //历史版本和已经删除的文件不会冲突
        return Ok(DfsFileAction::Insert);
    }
    let Some(exists) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, row.user_id, row.parent_id, &row.name)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(DfsFileAction::Insert);
    };
    let err = |e: sqlx::Error| e.to_string();
    if exists.id == row.id {
        return Ok(DfsFileAction::Skip);
    }
    if row.storage_id == 0 && exists.storageId == 0 {
        if row.id < exists.id {
            merge_folder(ctx, exists.id, row.id).await?;
            return Ok(DfsFileAction::Insert);
        }
        alias::add(ctx, row.id, exists.id, "dfs_file").await.map_err(err)?;
        return Ok(DfsFileAction::Skip);
    }
    if row.storage_id > 0 && exists.storageId > 0 {
        if row.is_extra || exists.isExtra == 1 {
            if row.id < exists.id {
                table_data::execute(ctx, "delete from dfs_file where id = ?", &[exists.id.into()]).await.map_err(err)?;
                return Ok(DfsFileAction::Insert);
            }
            return Ok(DfsFileAction::Skip);
        }
        if row.id > exists.id {
            //主机端的文件比较新
            table_data::execute(ctx, "update dfs_file set isHistory = 1 where id = ?", &[exists.id.into()]).await.map_err(err)?;
            return Ok(DfsFileAction::Insert);
        }
        return Ok(DfsFileAction::InsertAsHistory);
    }
    Err(conflict_msg(ctx, info, row.user_id, exists.id).await)
}

/// 同名的文件和文件夹冲突时的提示
pub async fn conflict_msg(ctx: &mut DbContext, info: &SyncInfo, user_id: i64, exists_id: i64) -> String {
    let user_name = user_dao::select_one(&mut *ctx, user_id)
        .await
        .ok()
        .and_then(|it| it.name)
        .unwrap_or_default();
    let path = dfs_file_service::get_path_by_id(ctx, exists_id).await.unwrap_or_default();
    format!(
        "同步失败，服务器：{}  用户名：{}  路径：{} 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请在本机将 {} 重命名，然后点击重新同步。",
        info.url.split('/').take(3).collect::<Vec<_>>().join("/"),
        user_name,
        path,
        path
    )
}

/// 合并两个同名文件夹:被合并的文件夹下的内容移到保留的文件夹下,然后删除被合并的文件夹
/// 保留的文件夹在本机可能还不存在(马上要添加)
/// - drop_id 被合并的文件夹
/// - keep_id 保留的文件夹(ID比较小的)
pub async fn merge_folder(ctx: &mut DbContext, drop_id: i64, keep_id: i64) -> Result<(), String> {
    let err = |e: sqlx::Error| e.to_string();

    //被合并的文件夹本身可能已经不在了(全量同步时只同步到了它下面的文件)
    let user_id = match dfs_file_dao::select_one(&mut *ctx, drop_id).await {
        Ok(it) => Some(it.userId),
        Err(_) => dfs_file_dao::select_all_child_list(&mut *ctx, drop_id).await.map_err(err)?.first().map(|it| it.userId),
    };
    let children = match user_id {
        Some(user_id) => dfs_file_dao::select_sub_list(&mut *ctx, user_id, drop_id).await.map_err(err)?,
        None => Vec::new(),
    };

    //两个文件夹下有同名的文件或文件夹时,按同样的规则处理
    for child in children {
        let Some(other) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, child.userId, keep_id, &child.name).await.map_err(err)? else {
            continue;
        };
        match (child.storageId == 0, other.storageId == 0) {
            (true, true) => {
                if child.id < other.id {
                    Box::pin(merge_folder(ctx, other.id, child.id)).await?;
                } else {
                    Box::pin(merge_folder(ctx, child.id, other.id)).await?;
                }
            }
            (false, false) => {
                let older = child.id.min(other.id);
                table_data::execute(ctx, "update dfs_file set isHistory = 1 where id = ?", &[older.into()]).await.map_err(err)?;
            }
            _ => {
                let path = dfs_file_service::get_path_by_id(ctx, other.id).await.unwrap_or_default();
                return Err(format!("合并同名文件夹失败，路径：{} 同一个文件夹下有同名的文件和文件夹，请重命名之后重新同步。", path));
            }
        }
    }

    //剩下的(包括历史版本和回收站里的)全部移过去
    table_data::execute(ctx, "update dfs_file set parentId = ? where parentId = ?", &[keep_id.into(), drop_id.into()]).await.map_err(err)?;

    //子账户的授权也换成保留的文件夹,同一个子账户都有授权时保留ID小的授权
    let grants = table_data::select(ctx, "select id, subUserId from sub_account_grant where folderId = ?", &[drop_id.into()]).await.map_err(err)?;
    for grant in grants {
        let id = grant.get("id").and_then(|it| it.as_i64()).unwrap_or(0);
        let sub_user_id = grant.get("subUserId").and_then(|it| it.as_i64()).unwrap_or(0);
        let dup = table_data::select(
            ctx,
            "select id from sub_account_grant where subUserId = ? and folderId = ?",
            &[sub_user_id.into(), keep_id.into()],
        )
        .await
        .map_err(err)?;
        match dup.first().and_then(|it| it.get("id")).and_then(|it| it.as_i64()) {
            Some(dup_id) if dup_id < id => {
                table_data::execute(ctx, "delete from sub_account_grant where id = ?", &[id.into()]).await.map_err(err)?;
            }
            dup_id => {
                if let Some(dup_id) = dup_id {
                    table_data::execute(ctx, "delete from sub_account_grant where id = ?", &[dup_id.into()]).await.map_err(err)?;
                }
                table_data::execute(ctx, "update sub_account_grant set folderId = ? where id = ?", &[keep_id.into(), id.into()]).await.map_err(err)?;
            }
        }
    }
    table_data::execute(ctx, "delete from dfs_file where id = ?", &[drop_id.into()]).await.map_err(err)?;
    alias::add(ctx, drop_id, keep_id, "dfs_file").await.map_err(err)?;
    Ok(())
}

/// 同步子账户的授权之前,处理同一个子账户对同一个文件夹已经有授权的情况(两台服务器同时添加了授权),保留ID小的
/// - return 是否添加
pub async fn grant(ctx: &mut DbContext, id: i64, sub_user_id: i64, folder_id: i64) -> Result<bool, String> {
    let err = |e: sqlx::Error| e.to_string();
    let exists = table_data::select(
        ctx,
        "select id from sub_account_grant where subUserId = ? and folderId = ?",
        &[sub_user_id.into(), folder_id.into()],
    )
    .await
    .map_err(err)?;
    match exists.first().and_then(|it| it.get("id")).and_then(|it| it.as_i64()) {
        None => Ok(true),
        Some(exists_id) if exists_id == id => Ok(false),
        Some(exists_id) if id < exists_id => {
            table_data::execute(ctx, "delete from sub_account_grant where id = ?", &[exists_id.into()]).await.map_err(err)?;
            Ok(true)
        }
        Some(_) => Ok(false),
    }
}

/// 修改文件位置(移动、重命名)之前,处理目标位置已经有同名文件的情况(两台服务器同时把文件移动或重命名成同一个名字)
/// 和添加文件时的规则一样
/// - return 是否继续修改
pub async fn move_conflict(ctx: &mut DbContext, info: &SyncInfo, id: i64, parent_id: i64, name: &str) -> Result<bool, String> {
    let err = |e: sqlx::Error| e.to_string();
    let Ok(file) = dfs_file_dao::select_one(&mut *ctx, id).await else {
        return Ok(true);
    };
    if file.isHistory == 1 || file.deleteDate.is_some() || file.isExtra == 1 {
        return Ok(true);
    }
    let Some(exists) = dfs_file_dao::select_by_parent_id_and_name(&mut *ctx, file.userId, parent_id, name).await.map_err(err)? else {
        return Ok(true);
    };
    if exists.id == id {
        return Ok(true);
    }
    match (file.storageId == 0, exists.storageId == 0) {
        (true, true) => {
            if id < exists.id {
                merge_folder(ctx, exists.id, id).await?;
                Ok(true)
            } else {
                merge_folder(ctx, id, exists.id).await?;
                Ok(false)
            }
        }
        (false, false) => {
            let older = id.min(exists.id);
            table_data::execute(ctx, "update dfs_file set isHistory = 1 where id = ?", &[older.into()]).await.map_err(err)?;
            Ok(true)
        }
        _ => Err(conflict_msg(ctx, info, file.userId, exists.id).await),
    }
}

/// 添加用户文件时需要用到的字段
#[derive(Debug)]
pub struct DfsFileRow {
    pub id: i64,
    pub user_id: i64,
    pub parent_id: i64,
    pub name: String,
    pub storage_id: i64,
    pub is_extra: bool,
    pub is_history: bool,
    pub is_deleted: bool,
}

impl DfsFileRow {
    /// 从全量同步的数据中读取
    pub fn from_map(row: &serde_json::Map<String, Value>) -> Self {
        let int = |key: &str| row.get(key).and_then(|it| it.as_i64()).unwrap_or(0);
        Self {
            id: int("id"),
            user_id: int("userId"),
            parent_id: int("parentId"),
            name: row.get("name").and_then(|it| it.as_str()).unwrap_or_default().to_string(),
            storage_id: int("storageId"),
            is_extra: int("isExtra") == 1,
            is_history: int("isHistory") == 1,
            is_deleted: row.get("deleteDate").is_some_and(|it| !it.is_null()),
        }
    }

    /// 从同步日志的insert语句参数中读取(参数顺序与dfs_file表的列顺序一致)
    pub fn from_params(params: &[Value]) -> Option<Self> {
        let int = |i: usize| params.get(i).and_then(|it| it.as_i64()).unwrap_or(0);
        Some(Self {
            id: params.first()?.as_i64()?,
            user_id: int(1),
            parent_id: int(2),
            name: params.get(3)?.as_str()?.to_string(),
            storage_id: int(7),
            is_extra: int(10) == 1,
            is_history: int(11) == 1,
            is_deleted: params.get(12).is_some_and(|it| !it.is_null()),
        })
    }
}
