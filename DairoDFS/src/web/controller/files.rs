use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::{dfs_file_dao, storage_file_dao};
use crate::extension::number::NumberExtension;
use crate::extension::string::StringExtension;
use crate::service::{dfs_file_service, file_handle_service};
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{biz_error, biz_errorf, db};
use hyper::HeaderMap;
use crate::util::media;
use crate::util::media::image_util;
use hyperoute_core::{Ext, Raw, Stream};
use std::path::Path;
use hyperoute_macro::{DELETE, GET, POST, PUT};
use std::collections::HashSet;

/// 获取文件夹下的文件列表
/// - folder 文件夹路径,空字符串代表根目录
#[GET("/api/files/list")]
pub async fn list(login: Ext<LoginUser>, folder: String) -> Result<model::FileList, BizError> {
    let mut ctx = lib_db::get_context();
    let Some(folder_id) = dfs_file_service::get_folder_id(&mut ctx, login.id, &folder, false).await? else {
        return Err(biz_error!("文件夹不存在"));
    };
    let thumb_ids: HashSet<i64> = dfs_file_dao::select_has_thumb_ids(&mut ctx, login.id, folder_id)
        .await?
        .into_iter()
        .collect();
    let list = dfs_file_dao::select_sub_list(&mut ctx, login.id, folder_id)
        .await?
        .into_iter()
        .map(|it| model::FileItem {
            id: it.id,
            has_thumb: thumb_ids.contains(&it.id),
            name: it.name,
            size: it.size,
            date: it.date,
            file_flag: it.storageId > 0,
        })
        .collect();
    Ok(model::FileList { folder_id, list })
}

/// 创建文件夹
/// - folder 文件夹路径
#[POST("/api/files/create_folder")]
pub async fn create_folder(login: Ext<LoginUser>, folder: String) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    if dfs_file_service::select_by_path(&mut ctx, login.id, &folder).await?.is_some() {
        return Err(biz_errorf!("文件或文件夹[{}]已存在", folder));
    }
    dfs_file_service::mkdirs(&mut ctx, login.id, &folder).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 重命名
#[PUT("/api/files/rename")]
pub async fn rename(login: Ext<LoginUser>, id: i64, name: String) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_service::rename(&mut ctx, login.id, id, &name).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 复制
/// - ids 要复制的文件ID
/// - target_folder 目标文件夹路径
/// - is_over_write 是否覆盖目标文件
#[POST("/api/files/copy")]
pub async fn copy(login: Ext<LoginUser>, ids: Vec<i64>, target_folder: String, is_over_write: bool) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_service::copy(&mut ctx, login.id, &ids, &target_folder, is_over_write).await?;
    db::commit(ctx).await?;

    //复制出来的文件需要重新生成附属文件(同一个存储文件已经生成过的会直接复用)
    file_handle_service::notify();
    Ok(())
}

/// 移动
/// - ids 要移动的文件ID
/// - target_folder 目标文件夹路径
/// - is_over_write 是否覆盖目标文件
#[POST("/api/files/move")]
pub async fn move_to(login: Ext<LoginUser>, ids: Vec<i64>, target_folder: String, is_over_write: bool) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_service::move_to(&mut ctx, login.id, &ids, &target_folder, is_over_write).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 删除到垃圾箱
#[DELETE("/api/files/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    for id in ids {
        dfs_file_service::set_delete(&mut ctx, login.id, id).await?;
    }
    db::commit(ctx).await?;
    Ok(())
}

/// 文件或文件夹属性
/// - ids 选中的文件ID,选中多个时只统计数量和大小
#[GET("/api/files/property")]
pub async fn property(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<model::FileProperty, BizError> {
    let mut ctx = lib_db::get_context();
    let mut out = model::FileProperty::default();
    if ids.len() != 1 {
        let mut total = model::SubTotal::default();
        for id in ids {
            let file = dfs_file_service::select_own(&mut ctx, login.id, id).await?;
            if file.storageId > 0 {
                total.file_count += 1;
                total.size += file.size;
            } else {
                total.folder_count += 1;
                compute_sub_total(&mut ctx, &file, &mut total).await?;
            }
        }
        out.size = total.size.data_size();
        out.file_count = Some(total.file_count);
        out.folder_count = Some(total.folder_count);
        return Ok(out);
    }
    let file = dfs_file_service::select_own(&mut ctx, login.id, ids[0]).await?;
    out.id = file.id;
    out.path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
    out.date = file.date.date_format();
    out.is_file = file.storageId > 0;
    out.state = file.state;
    out.state_msg = file.stateMsg.clone().unwrap_or_default();
    if file.storageId == 0 {
        let mut total = model::SubTotal::default();
        compute_sub_total(&mut ctx, &file, &mut total).await?;
        out.name = file.name;
        out.size = total.size.data_size();
        out.file_count = Some(total.file_count);
        out.folder_count = Some(total.folder_count);
        return Ok(out);
    }
    out.history_list = dfs_file_dao::select_history(&mut ctx, login.id, file.parentId, &file.name)
        .await?
        .into_iter()
        .map(|it| model::FileHistory {
            id: it.id,
            size: it.size.data_size(),
            date: it.date.date_format(),
        })
        .collect();
    out.extra_list = dfs_file_dao::select_extra_list(&mut ctx, file.id)
        .await?
        .into_iter()
        .map(|it| model::FileExtra {
            id: it.id,
            name: it.name,
            size: it.size.data_size(),
            content_type: it.contentType.unwrap_or_default(),
        })
        .collect();
    out.name = file.name;
    out.size = file.size.data_size();
    out.content_type = file.contentType.unwrap_or_default();
    out.property = file.property.unwrap_or_default();
    Ok(out)
}

/// 统计文件夹下的文件数、文件夹数及文件总大小
async fn compute_sub_total(ctx: &mut sqlx_context::DbContext, folder: &DfsFile, total: &mut model::SubTotal) -> Result<(), BizError> {
    let mut folder_ids = vec![folder.id];
    while let Some(folder_id) = folder_ids.pop() {
        for it in dfs_file_dao::select_sub_list(&mut *ctx, folder.userId, folder_id).await? {
            if it.storageId > 0 {
                total.file_count += 1;
                total.size += it.size;
            } else {
                total.folder_count += 1;
                folder_ids.push(it.id);
            }
        }
    }
    Ok(())
}

/// 修改文件类型
#[PUT("/api/files/content_type")]
pub async fn content_type(login: Ext<LoginUser>, id: i64, content_type: String) -> Result<(), BizError> {
    let content_type = content_type.trim();
    if content_type.is_empty() {
        return Err(biz_error!("文件类型不能为空"));
    }
    let mut ctx = db::begin().await?;
    dfs_file_service::select_own(&mut ctx, login.id, id).await?;
    dfs_file_dao::set_content_type(&mut ctx, id, content_type).await?;
    db::commit(ctx).await?;
    Ok(())
}

/// 下载文件(包括历史版本)
/// - name 文件名,仅用于浏览器显示和保存时的文件名
/// - download 是否以附件的方式下载
#[GET("/api/files/download/{id}/{name}")]
pub async fn download(login: Ext<LoginUser>, id: i64, name: String, download: Option<bool>, headers: &HeaderMap) -> Result<Stream, BizError> {
    let mut ctx = lib_db::get_context();
    let Ok(file) = dfs_file_service::select_own(&mut ctx, login.id, id).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 || file.isExtra == 1 {
        return Err(BizError::not_found());
    }
    let stream = file_stream(&file, headers).await?;
    if download == Some(true) {
        return Ok(stream.attachment(&name));
    }
    Ok(stream)
}

/// 获取附属文件,如缩略图、预览图、转码后的视频等
/// - id 文件ID
/// - extra 附属文件名
#[GET("/api/files/extra/{id}/{extra}")]
pub async fn extra(login: Ext<LoginUser>, id: i64, extra: String, headers: &HeaderMap) -> Result<Stream, BizError> {
    let mut ctx = lib_db::get_context();
    if dfs_file_service::select_own(&mut ctx, login.id, id).await.is_err() {
        return Err(BizError::not_found());
    }
    let Some(extra_file) = dfs_file_dao::select_extra(&mut ctx, id, &extra).await? else {
        return Err(BizError::not_found());
    };
    file_stream(&extra_file, headers).await
}

/// 获取缩略图
#[GET("/api/files/thumb/{id}")]
pub async fn thumb(login: Ext<LoginUser>, id: i64, headers: &HeaderMap) -> Result<Stream, BizError> {
    extra(login, id, "thumb".to_string(), headers).await
}

/// 在线生成缩略图同时只允许一个,避免大量请求时占满CPU和内存
static THUMB_ONLINE_LOCK: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

/// 在线生成缩略图,优先使用max_size
/// - max_size 最大边
/// - width,height 按宽高裁剪
/// - name 文件名,指定时以附件的方式下载
#[GET("/api/files/thumb_online/{id}")]
pub async fn thumb_online(
    login: Ext<LoginUser>,
    id: i64,
    max_size: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    name: Option<String>,
) -> Result<Raw, BizError> {
    let mut ctx = lib_db::get_context();
    let Ok(file) = dfs_file_service::select_own(&mut ctx, login.id, id).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 {
        return Err(BizError::not_found());
    }

    //有缩略图时用缩略图生成,否则用原文件的预览图
    let source = match dfs_file_dao::select_extra(&mut ctx, id, "thumb").await? {
        Some(thumb) => {
            let storage = storage_file_dao::select_one(&mut ctx, thumb.storageId).await?;
            tokio::fs::read(&storage.path).await.map_err(|_| BizError::not_found())?
        }
        None => {
            let storage = storage_file_dao::select_one(&mut ctx, file.storageId).await?;
            media::preview_jpg(&file.ext, Path::new(&storage.path))
                .await
                .map_err(BizError::msg)?
                .ok_or_else(BizError::not_found)?
        }
    };
    let _permit = THUMB_ONLINE_LOCK.acquire().await.unwrap();
    let data = match (max_size, width, height) {
        (Some(max_size), _, _) if max_size > 0 => media::blocking(move || image_util::resize(&source, max_size, 85)).await,
        (_, Some(width), Some(height)) if width > 0 && height > 0 => {
            media::blocking(move || image_util::crop(&source, width, height, 85)).await
        }
        _ => return Err(biz_error!("width,height和maxSize至少要传其中一种参数")),
    }
    .map_err(BizError::msg)?;
    let mut raw = Raw::new(data)
        .header("content-type", "image/jpeg")
        .header("cache-control", "private, max-age=31536000, immutable");
    if let Some(name) = name.filter(|it| !it.is_empty()) {
        raw = raw.header("content-disposition", hyperoute_core::content_disposition("attachment", &name));
    }
    Ok(raw)
}

/// 获取相册列表(所有图片和视频),按拍摄时间升序
/// 每一项为 [文件ID, 拍摄时间(秒), 扩展名, 是否有缩略图, 视频时长(毫秒)]
/// 数据量可能很大,所以使用数组格式减小数据体积
#[GET("/api/files/album")]
pub async fn album(login: Ext<LoginUser>) -> Result<Vec<(i64, i64, String, bool, i64)>, BizError> {
    let mut ctx = lib_db::get_context();
    let thumb_ids: HashSet<i64> = dfs_file_dao::select_thumb_parent_ids(&mut ctx, login.id)
        .await?
        .into_iter()
        .collect();
    let mut list: Vec<(i64, i64, String, bool, i64)> = dfs_file_dao::select_album(&mut ctx, login.id)
        .await?
        .into_iter()
        .map(|it| {
            let property: serde_json::Value = it
                .property
                .as_deref()
                .and_then(|it| serde_json::from_str(it).ok())
                .unwrap_or_default();

            //没有拍摄时间时,使用上传时间
            let capture_time = property["date"].as_i64().filter(|it| *it > 0).unwrap_or(it.date);
            let duration = property["duration"].as_i64().unwrap_or(0);
            (it.id, capture_time / 1000, it.ext, thumb_ids.contains(&it.id), duration)
        })
        .collect();
    list.sort_by_key(|it| it.1);
    Ok(list)
}

/// 文件下载响应,支持断点续传
pub async fn file_stream(file: &DfsFile, headers: &HeaderMap) -> Result<Stream, BizError> {
    let Ok(storage) = storage_file_dao::select_one(&mut lib_db::get_context(), file.storageId).await else {
        return Err(BizError::not_found());
    };
    //为了防止暴露文件真正的MD5，这里将MD5再次加密
    let md5 = storage.md5.md5();

    //ETag按文件内容生成,集群里所有服务器都一样,客户端切换服务器之后也能断点续传
    let Ok(stream) = Stream::file_with_etag(&storage.path, headers, &md5).await else {
        return Err(BizError::not_found());
    };
    let mut stream = stream
        //文件ID对应的内容不会改变,允许浏览器长时间缓存
        .set_header("cache-control", "private, max-age=31536000, immutable")
        .set_header("content-md5", md5);
    if let Some(content_type) = &file.contentType {
        stream = stream.content_type(content_type.as_str());
    }
    Ok(stream)
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileList {

        // 当前文件夹ID
        pub folder_id: i64,

        // 文件列表
        pub list: Vec<FileItem>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileItem {

        // 文件ID
        pub id: i64,

        // 文件名
        pub name: String,

        // 文件大小
        pub size: i64,

        // 创建日期(时间戳毫秒)
        pub date: i64,

        // 是否文件
        pub file_flag: bool,

        // 是否有缩略图
        pub has_thumb: bool,
    }

    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileProperty {

        // 文件ID(单个文件时)
        pub id: i64,

        // 名称
        pub name: String,

        // 路径
        pub path: String,

        // 大小
        pub size: String,

        // 创建日期
        pub date: String,

        // 是否文件
        pub is_file: bool,

        // 文件类型
        pub content_type: String,

        // 文件属性(图片尺寸、视频时长等,JSON字符串)
        pub property: String,

        // 文件处理状态 0：待处理 1：处理完成 2：处理出错
        pub state: i64,

        // 文件处理信息(出错时为错误信息)
        pub state_msg: String,

        // 文件数(文件夹或多选时)
        pub file_count: Option<i64>,

        // 文件夹数(文件夹或多选时)
        pub folder_count: Option<i64>,

        // 历史版本
        pub history_list: Vec<FileHistory>,

        // 附属文件
        pub extra_list: Vec<FileExtra>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileHistory {

        // 文件ID
        pub id: i64,

        // 大小
        pub size: String,

        // 日期
        pub date: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FileExtra {

        // 文件ID
        pub id: i64,

        // 附属文件名
        pub name: String,

        // 大小
        pub size: String,

        // 文件类型
        pub content_type: String,
    }

    /// 文件夹统计
    #[derive(Debug, Default)]
    pub struct SubTotal {

        // 文件数
        pub file_count: i64,

        // 文件夹数
        pub folder_count: i64,

        // 文件总大小
        pub size: i64,
    }
}
