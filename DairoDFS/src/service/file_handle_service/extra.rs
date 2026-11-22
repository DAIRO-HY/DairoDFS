//! 附属文件(缩略图、预览图、转码后的视频等)的读取和添加

use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::{dfs_file_dao, storage_file_dao};
use crate::db;
use crate::service::{dfs_file_service, storage_service};
use crate::util::{file_util, id_util};
use std::path::{Path, PathBuf};

/// 文件的本地存储路径
pub async fn storage_path(file: &DfsFile) -> Result<PathBuf, String> {
    let storage = storage_file_dao::select_one(&mut lib_db::get_context(), file.storageId)
        .await
        .map_err(|_| format!("本地存储文件:{}不存在", file.storageId))?;
    let path = PathBuf::from(&storage.path);
    if !path.is_file() {
        return Err(format!("文件:{}不存在", storage.path));
    }
    Ok(path)
}

/// 获取文件已经有的附属文件
pub async fn get(file: &DfsFile, name: &str) -> Result<Option<DfsFile>, String> {
    dfs_file_dao::select_extra(&mut lib_db::get_context(), file.id, name).await.map_err(|e| e.to_string())
}

/// 附属文件已经存在时跳过,同一个文件在其他地方已经生成过附属文件时直接复用
/// - return 是否已经存在或者复用成功
pub async fn reuse(file: &DfsFile, name: &str) -> Result<bool, String> {
    if get(file, name).await?.is_some() {
        return Ok(true);
    }
    let Some(exists) = dfs_file_dao::select_extra_by_storage_id_and_name(&mut lib_db::get_context(), file.storageId, name)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(false);
    };
    add(file, name, exists.storageId, exists.size, exists.contentType.as_deref().unwrap_or_default()).await?;
    Ok(true)
}

/// 保存数据并添加附属文件
pub async fn add_data(file: &DfsFile, name: &str, data: &[u8], content_type: &str) -> Result<(), String> {
    let storage = storage_service::save_data(data).await.map_err(|e| e.to_string())?;
    add(file, name, storage.id, data.len() as i64, content_type).await
}

/// 保存转码后的视频(临时文件)并添加附属文件
pub async fn add_video(file: &DfsFile, name: &str, path: &Path) -> Result<(), String> {
    let md5 = file_util::md5(path).await.map_err(|e| e.to_string())?;
    let size = tokio::fs::metadata(path).await.map_err(|e| e.to_string())?.len() as i64;
    let storage = storage_service::save(path, &md5).await.map_err(|e| e.to_string())?;
    add(file, name, storage.id, size, "video/mp4").await
}

/// 添加附属文件
pub async fn add(file: &DfsFile, name: &str, storage_id: i64, size: i64, content_type: &str) -> Result<(), String> {
    let extra = DfsFile {
        id: id_util::id(),
        userId: file.userId,
        parentId: file.id,
        name: name.to_string(),
        size,
        contentType: Some(content_type.to_string()),
        storageId: storage_id,
        date: file.date,
        isExtra: 1,
        state: 1,
        ..Default::default()
    };
    let mut ctx = db::begin().await.map_err(|e| e.to_string())?;
    dfs_file_service::check_storage(&mut ctx, storage_id).await.map_err(|_| "存储文件不存在".to_string())?;
    dfs_file_dao::insert(&mut ctx, &extra).await.map_err(|e| e.to_string())?;
    db::commit(ctx).await.map_err(|e| e.to_string())
}
