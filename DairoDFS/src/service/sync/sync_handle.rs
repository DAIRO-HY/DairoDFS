use crate::application;
use crate::dao::{dfs_file_dao, storage_file_dao, user_dao};
use crate::extension::number::NumberExtension;
use crate::service::sync::sync_http::DOWNLOAD_CLIENT;
use crate::service::sync::sync_info::SyncInfo;
use crate::service::sync::table_data;
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

/// 同步存储文件(在事务中执行)
/// - prepared prepare_storage_file下载的文件路径
/// - return 本机的存储路径
pub async fn storage_file(ctx: &mut DbContext, md5: &str, master_id: i64, prepared: Option<String>) -> Result<String, String> {
    let exists = storage_file_dao::select_by_md5(&mut *ctx, md5).await.map_err(|e| e.to_string())?;
    match exists {
        //文件已经存在,并且与主机端的ID一样(全量同步时用到)
        Some(it) if it.id == master_id => Ok(it.path),

        //本机存在同样的文件,直接使用,并把本机的存储ID换成主机的ID
        Some(it) => {
            let err = |e: sqlx::Error| e.to_string();
            table_data::execute(ctx, "delete from storage_file where id = ?", &[it.id.into()]).await.map_err(err)?;
            table_data::execute(ctx, "update dfs_file set storageId = ? where storageId = ?", &[master_id.into(), it.id.into()]).await.map_err(err)?;
            table_data::execute(ctx, "update dfs_file_delete set storageId = ? where storageId = ?", &[master_id.into(), it.id.into()]).await.map_err(err)?;
            Ok(it.path)
        }
        None => prepared.ok_or_else(|| format!("文件{}没有下载", md5)),
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
        //文件已经下载完成
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

/// 添加用户文件之前,处理本机已经存在同名文件的情况
/// - 都是文件夹: 保留主机端的文件夹,本机文件夹下的内容移到主机端的文件夹下
/// - 都是附属文件: 删除本机生成的附属文件,以主机端为准
/// - 都是文件: 保留比较新的文件,另一个设置为历史版本
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
        table_data::execute(ctx, "update dfs_file set parentId = ? where parentId = ?", &[row.id.into(), exists.id.into()]).await.map_err(err)?;
        table_data::execute(ctx, "delete from dfs_file where id = ?", &[exists.id.into()]).await.map_err(err)?;
        return Ok(DfsFileAction::Insert);
    }
    if row.storage_id > 0 && exists.storageId > 0 {
        if row.is_extra || exists.isExtra == 1 {
            table_data::execute(ctx, "delete from dfs_file where id = ?", &[exists.id.into()]).await.map_err(err)?;
            return Ok(DfsFileAction::Insert);
        }
        if row.id > exists.id {
            //主机端的文件比较新
            table_data::execute(ctx, "update dfs_file set isHistory = 1 where id = ?", &[exists.id.into()]).await.map_err(err)?;
            return Ok(DfsFileAction::Insert);
        }
        return Ok(DfsFileAction::InsertAsHistory);
    }
    let user_name = user_dao::select_one(&mut *ctx, row.user_id)
        .await
        .ok()
        .and_then(|it| it.name)
        .unwrap_or_default();
    let path = dfs_file_service::get_path_by_id(ctx, exists.id).await.unwrap_or_default();
    Err(format!(
        "同步失败，服务器：{}  用户名：{}  路径：{} 文件冲突。原因：同一个文件夹下，不允许同名的文件或文件夹。解决方案：请重命名当前或者服务器端 {} 的文件名。",
        info.url.split('/').take(3).collect::<Vec<_>>().join("/"),
        user_name,
        path,
        path
    ))
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
