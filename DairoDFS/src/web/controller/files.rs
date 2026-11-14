use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::{dfs_file_dao, storage_file_dao};
use crate::extension::number::NumberExtension;
use crate::extension::string::StringExtension;
use crate::service::operate_log_service::{self, op};
use crate::service::sub_account_service::{perm, Scope};
use crate::service::{dfs_file_service, file_handle_service};
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{biz_error, biz_errorf, db};
use hyper::{HeaderMap, StatusCode};
use crate::util::media;
use crate::util::media::image_util;
use crate::util::request_util::{self, ByteRange};
use hyperoute_core::{Ext, Raw, Stream};
use hyperoute_macro::{DELETE, GET, POST, PUT};
use std::collections::HashSet;

/// 获取文件夹下的文件列表
/// - folder 文件夹路径,空字符串代表根目录
#[GET("/api/files/list")]
pub async fn list(login: Ext<LoginUser>, folder: String) -> Result<model::FileList, BizError> {
    let mut ctx = lib_db::get_context();
    let (scope, folder_id, folder_perms) = list_folder(&mut ctx, &login, &folder).await?;
    let thumb_ids: HashSet<i64> = dfs_file_dao::select_has_thumb_ids(&mut ctx, login.owner_id, folder_id)
        .await?
        .into_iter()
        .collect();
    let list = dfs_file_dao::select_list_items(&mut ctx, login.owner_id, folder_id)
        .await?
        .into_iter()
        .filter(|it| scope.visible(folder_perms, it.id))
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

/// 获取要显示列表的文件夹,并检查浏览权限
/// - return 访问范围、文件夹ID、对文件夹的权限
pub async fn list_folder(ctx: &mut sqlx_context::DbContext, login: &LoginUser, folder: &str) -> Result<(Scope, i64, i64), BizError> {
    let mut scope = Scope::load(ctx, login).await?;
    let folder = match dfs_file_service::select_by_path(ctx, login.owner_id, folder).await? {
        Some(it) if it.storageId == 0 => it,
        _ => return Err(biz_error!("文件夹不存在")),
    };
    let perms = scope.perms(ctx, &folder).await?;
    if perms & (perm::VIEW | perm::PASS) == 0 {
        return Err(biz_error!("文件夹不存在"));
    }
    Ok((scope, folder.id, perms))
}

/// 创建文件夹
/// - folder 文件夹路径
#[POST("/api/files/create_folder")]
pub async fn create_folder(login: Ext<LoginUser>, folder: String) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    if dfs_file_service::select_by_path(&mut ctx, login.owner_id, &folder).await?.is_some() {
        return Err(biz_errorf!("文件或文件夹[{}]已存在", folder));
    }
    Scope::load(&mut ctx, &login).await?.check_path(&mut ctx, login.owner_id, &folder, perm::UPLOAD).await?;
    let id = dfs_file_service::mkdirs(&mut ctx, login.owner_id, &folder).await?;
    db::commit(ctx).await?;
    operate_log_service::add(&login, op::CREATE_FOLDER, Some(id), Some(folder), None);
    Ok(())
}

/// 重命名
#[PUT("/api/files/rename")]
pub async fn rename(login: Ext<LoginUser>, id: i64, name: String) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, id).await?;
    Scope::load(&mut ctx, &login).await?.check_in_parent(&mut ctx, &file, perm::MODIFY).await?;
    let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
    dfs_file_service::rename(&mut ctx, login.owner_id, id, &name).await?;
    db::commit(ctx).await?;
    operate_log_service::add_file(&login, op::RENAME, &file, path, Some(serde_json::json!({"name": name})));
    Ok(())
}

/// 复制
/// - ids 要复制的文件ID
/// - target_folder 目标文件夹路径
/// - is_over_write 是否覆盖目标文件
#[POST("/api/files/copy")]
pub async fn copy(login: Ext<LoginUser>, ids: Vec<i64>, target_folder: String, is_over_write: bool) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut files = Vec::new();
    for id in &ids {
        let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, *id).await?;
        scope.check(&mut ctx, &file, perm::MODIFY).await?;
        let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
        files.push((file, path));
    }
    check_target(&mut ctx, &mut scope, &login, &target_folder, is_over_write).await?;
    let creator_id = login.is_sub.then_some(login.id);
    dfs_file_service::copy(&mut ctx, login.owner_id, &ids, &target_folder, is_over_write, creator_id).await?;
    db::commit(ctx).await?;
    let detail = serde_json::json!({"target": target_folder, "overWrite": is_over_write});
    for (file, path) in files {
        operate_log_service::add_file(&login, op::COPY, &file, path, Some(detail.clone()));
    }

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
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut files = Vec::new();
    for id in &ids {
        let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, *id).await?;
        scope.check_in_parent(&mut ctx, &file, perm::MODIFY).await?;
        let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
        files.push((file, path));
    }
    check_target(&mut ctx, &mut scope, &login, &target_folder, is_over_write).await?;
    dfs_file_service::move_to(&mut ctx, login.owner_id, &ids, &target_folder, is_over_write).await?;
    db::commit(ctx).await?;
    let detail = serde_json::json!({"target": target_folder, "overWrite": is_over_write});
    for (file, path) in files {
        operate_log_service::add_file(&login, op::MOVE, &file, path, Some(detail.clone()));
    }
    Ok(())
}

/// 检查复制、移动的目标文件夹的权限:需要上传权限,覆盖目标文件时还需要修改权限
async fn check_target(
    ctx: &mut sqlx_context::DbContext,
    scope: &mut Scope,
    login: &LoginUser,
    target_folder: &str,
    is_over_write: bool,
) -> Result<(), BizError> {
    scope.check_path(ctx, login.owner_id, target_folder, perm::UPLOAD).await?;
    if is_over_write {
        scope.check_path(ctx, login.owner_id, target_folder, perm::MODIFY).await?;
    }
    Ok(())
}

/// 删除到垃圾箱
#[DELETE("/api/files/delete")]
pub async fn delete(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<(), BizError> {
    let mut ctx = db::begin().await?;
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut files = Vec::new();
    for id in ids {
        let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, id).await?;
        scope.check_in_parent(&mut ctx, &file, perm::DELETE).await?;
        let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
        dfs_file_service::set_delete(&mut ctx, login.owner_id, id).await?;
        files.push((file, path));
    }
    db::commit(ctx).await?;
    for (file, path) in files {
        operate_log_service::add_file(&login, op::DELETE, &file, path, None);
    }
    Ok(())
}

/// 按路径删除文件到垃圾箱(其他程序只保存了路径、没有保存文件ID时使用,比如DairoMUSIC换头像后删除旧图片)
/// 和按ID删除一样,同名的历史版本一起移到垃圾箱;路径不存在时跳过,重复调用是安全的
/// 只能删除文件,路径是文件夹时整个请求返回400,一个文件都不删除,避免路径传错把整个文件夹删掉
/// - paths 文件的完整路径
#[DELETE("/api/files/delete_by_path")]
pub async fn delete_by_path(login: Ext<LoginUser>, paths: Vec<String>) -> Result<model::DeleteByPath, BizError> {
    if paths.is_empty() {
        return Err(BizError::param("paths", "请指定要删除的文件路径"));
    }
    let mut ctx = db::begin().await?;
    let mut scope = Scope::load(&mut ctx, &login).await?;

    //先检查所有路径,有一个不能删除时一个都不删除
    let mut files = Vec::new();
    let mut not_found = Vec::new();
    for path in paths.into_iter().collect::<std::collections::BTreeSet<_>>() {
        if let Err(e) = crate::util::file_util::to_sub_names(&path) {
            return Err(BizError::param("paths", &format!("{}:{}", path, e)));
        }
        let Some(file) = dfs_file_service::select_by_path(&mut ctx, login.owner_id, &path).await? else {
            not_found.push(path);
            continue;
        };
        if file.storageId == 0 {
            return Err(BizError::param("paths", &format!("{}是文件夹,只能删除文件", path)));
        }
        let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, file.id).await?;
        scope.check_in_parent(&mut ctx, &file, perm::DELETE).await?;
        files.push((file, path));
    }
    for (file, _) in &files {
        dfs_file_service::set_delete(&mut ctx, login.owner_id, file.id).await?;
    }
    db::commit(ctx).await?;
    let deleted = files.len();
    for (file, path) in files {
        operate_log_service::add_file(&login, op::DELETE, &file, path, None);
    }
    Ok(model::DeleteByPath { deleted, not_found })
}

/// 文件或文件夹属性
/// - ids 选中的文件ID,选中多个时只统计数量和大小
#[GET("/api/files/property")]
pub async fn property(login: Ext<LoginUser>, ids: Vec<i64>) -> Result<model::FileProperty, BizError> {
    //没有传ids时(比如参数名写成了id)提示错误,否则会被当成0个文件的合计返回空数据
    if ids.is_empty() {
        return Err(BizError::param("ids", "请选择文件"));
    }
    let mut ctx = lib_db::get_context();
    let mut scope = Scope::load(&mut ctx, &login).await?;
    let mut out = model::FileProperty::default();
    if ids.len() != 1 {
        let mut total = model::SubTotal::default();
        for id in ids {
            let file = dfs_file_service::select_own(&mut ctx, login.owner_id, id).await?;
            scope.check(&mut ctx, &file, perm::VIEW).await?;
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
    let file = dfs_file_service::select_own(&mut ctx, login.owner_id, ids[0]).await?;
    scope.check(&mut ctx, &file, perm::VIEW).await?;
    out.id = file.id;
    out.path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
    out.date = file.date.date_format();
    out.upload_date = file.date;
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
    out.history_list = dfs_file_dao::select_history(&mut ctx, login.owner_id, file.parentId, &file.name)
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
    let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, id).await?;
    Scope::load(&mut ctx, &login).await?.check(&mut ctx, &file, perm::MODIFY).await?;
    dfs_file_dao::set_content_type(&mut ctx, id, content_type).await?;
    let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
    db::commit(ctx).await?;
    operate_log_service::add_file(&login, op::CONTENT_TYPE, &file, path, Some(serde_json::json!({"contentType": content_type})));
    Ok(())
}

/// 修改拍摄时间,只修改文件属性里的拍摄时间,不修改文件本身
/// 第一次修改时,照片本身的拍摄时间保存在属性的originalDate里(没有拍摄时间时为0),可以恢复
/// - id 文件ID
/// - date 拍摄时间(时间戳毫秒),为0时恢复成照片本身的拍摄时间
/// - return 修改后的文件属性(和获取文件属性的接口一样)
#[PUT("/api/files/capture_time")]
pub async fn capture_time(login: Ext<LoginUser>, id: i64, date: i64) -> Result<model::FileProperty, BizError> {
    if date < 0 {
        return Err(biz_error!("拍摄时间不正确"));
    }
    let mut ctx = db::begin().await?;
    let file = dfs_file_service::select_operable(&mut ctx, login.owner_id, id).await?;
    Scope::load(&mut ctx, &login).await?.check(&mut ctx, &file, perm::MODIFY).await?;
    if file.storageId == 0 {
        return Err(biz_error!("文件夹不能修改拍摄时间"));
    }
    if file.state == 0 {
        //还没有生成属性,这时修改的话,生成属性时会被覆盖
        return Err(biz_error!("文件还在处理中,请稍后再试"));
    }
    let old_property = file.property.clone().unwrap_or_default();
    let new_property = if date == 0 {
        file_handle_service::restore_capture_time(&old_property)
    } else {
        //没有属性或者属性不是JSON对象时(比如不需要生成属性的文件),新建一个
        let mut map = match serde_json::from_str::<serde_json::Value>(&old_property) {
            Ok(serde_json::Value::Object(it)) => it,
            _ => serde_json::Map::new(),
        };
        if !map.contains_key(file_handle_service::ORIGINAL_DATE) {
            let original = map.get("date").cloned().unwrap_or(serde_json::json!(0));
            map.insert(file_handle_service::ORIGINAL_DATE.to_string(), original);
        }
        map.insert("date".to_string(), serde_json::json!(date));
        serde_json::Value::Object(map).to_string()
    };
    dfs_file_dao::set_property(&mut ctx, id, new_property).await?;
    let path = dfs_file_service::get_path_by_id(&mut ctx, file.id).await?;
    db::commit(ctx).await?;
    operate_log_service::add_file(&login, op::CAPTURE_TIME, &file, path, Some(serde_json::json!({"date": date})));
    property(login, vec![id]).await
}

/// 下载文件(包括历史版本)
/// - name 文件名,仅用于浏览器显示和保存时的文件名
/// - download 是否以附件的方式下载
#[GET("/api/files/download/{id}/{name}")]
pub async fn download(login: Ext<LoginUser>, id: i64, name: String, download: Option<bool>, headers: &HeaderMap) -> Result<Stream, BizError> {
    let mut ctx = lib_db::get_context();
    let Ok(file) = dfs_file_service::select_own(&mut ctx, login.owner_id, id).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 || file.isExtra == 1 {
        return Err(BizError::not_found());
    }
    check_file(&mut ctx, &login, &file, perm::OPEN).await?;
    let stream = file_stream(&file, headers).await?;
    operate_log_service::add_open(&login, &file, None).await;
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
    let Ok(file) = dfs_file_service::select_own(&mut ctx, login.owner_id, id).await else {
        return Err(BizError::not_found());
    };

    //缩略图只需要浏览权限,预览图、转码后的视频等需要打开权限
    check_file(&mut ctx, &login, &file, if extra == "thumb" { perm::VIEW } else { perm::OPEN }).await?;
    let Some(extra_file) = dfs_file_dao::select_extra(&mut ctx, id, &extra).await? else {
        return Err(BizError::not_found());
    };
    let stream = file_stream(&extra_file, headers).await?;
    if extra != "thumb" {
        //预览图、转码后的视频等也算打开了文件,缩略图不记录
        operate_log_service::add_open(&login, &file, Some(serde_json::json!({"extra": extra}))).await;
    }
    Ok(stream)
}

/// 获取缩略图
#[GET("/api/files/thumb/{id}")]
pub async fn thumb(login: Ext<LoginUser>, id: i64, headers: &HeaderMap) -> Result<Stream, BizError> {
    extra(login, id, "thumb".to_string(), headers).await
}

/// 在线生成缩略图同时只允许一个,避免大量请求时占满CPU和内存
/// 排队中的请求被客户端取消(断开连接)时,请求直接被丢弃,不再排队
static THUMB_ONLINE_LOCK: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

/// 在线生成缩略图的目标尺寸
#[derive(Clone, Copy)]
enum ThumbTarget {

    // 按比例缩放,最大边
    MaxSize(u32),

    // 按宽高从中间裁剪
    Crop(u32, u32),
}

impl ThumbTarget {

    /// 用这个尺寸的图片生成时,是否能达到目标尺寸
    fn fits(&self, width: u32, height: u32) -> bool {
        match *self {
            ThumbTarget::MaxSize(max_size) => width.max(height) >= max_size,
            ThumbTarget::Crop(target_width, target_height) => {
                //目标比图片更宽时,按图片的宽度裁剪,否则按图片的高度裁剪
                if target_width as u64 * height as u64 > target_height as u64 * width as u64 {
                    width >= target_width
                } else {
                    height >= target_height
                }
            }
        }
    }

    /// 生成图片,图片比目标尺寸小时不放大
    fn make(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        match *self {
            ThumbTarget::MaxSize(max_size) => image_util::resize(data, max_size, 85),
            ThumbTarget::Crop(width, height) => image_util::crop(data, width, height, 85),
        }
    }
}

/// 在线生成缩略图,优先使用max_size
/// 缩略图够大时用缩略图生成,否则用全尺寸的图片生成:
/// 普通图片用原文件,RAW、PSD、HEIC等用生成好的预览图,视频用生成好的首帧图,不会从原文件现场转换
/// - max_size 最大边
/// - width,height 按宽高裁剪
/// - name 文件名,指定时以附件的方式下载
/// 支持断点续传:不缓存生成的图片,每次都重新生成,只返回请求的那一段(同样的参数生成的图片完全一样)
#[GET("/api/files/thumb_online/{id}")]
pub async fn thumb_online(
    login: Ext<LoginUser>,
    id: i64,
    max_size: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    name: Option<String>,
    headers: &HeaderMap,
) -> Result<Raw, BizError> {
    let target = match (max_size, width, height) {
        (Some(max_size), _, _) if max_size > 0 => ThumbTarget::MaxSize(max_size),
        (_, Some(width), Some(height)) if width > 0 && height > 0 => ThumbTarget::Crop(width, height),
        _ => return Err(biz_error!("width,height和maxSize至少要传其中一种参数")),
    };
    let mut ctx = lib_db::get_context();
    let Ok(file) = dfs_file_service::select_own(&mut ctx, login.owner_id, id).await else {
        return Err(BizError::not_found());
    };
    if file.storageId == 0 {
        return Err(BizError::not_found());
    }
    check_file(&mut ctx, &login, &file, perm::VIEW).await?;
    let thumb_path = match dfs_file_dao::select_extra(&mut ctx, id, "thumb").await? {
        Some(thumb) => Some(storage_file_dao::select_one(&mut ctx, thumb.storageId).await?.path),
        None => None,
    };
    let full_storage_id = if media::is_image(&file.ext) {
        Some(file.storageId)
    } else {
        let extra_name = if media::is_video(&file.ext) { file_handle_service::PREVIEW_FRAME_NAME } else { "preview" };
        dfs_file_dao::select_extra(&mut ctx, id, extra_name).await?.map(|it| it.storageId)
    };
    let full_path = match full_storage_id {
        Some(storage_id) => Some(storage_file_dao::select_one(&mut ctx, storage_id).await?.path),
        None => None,
    };
    if thumb_path.is_none() && full_path.is_none() {
        return Err(BizError::not_found());
    }

    //排队时还没有读取图片,排队的请求不占内存
    let permit = THUMB_ONLINE_LOCK.acquire().await.unwrap();
    let data = media::blocking(move || {
        //生成完成之后才释放,请求在生成过程中被取消时,也要等这次生成完成,下一个请求才开始
        let _permit = permit;
        if let Some(thumb_path) = &thumb_path {
            let thumb = std::fs::read(thumb_path).map_err(|e| e.to_string())?;
            let fits = image_util::dimensions(&thumb).is_ok_and(|(width, height)| target.fits(width, height));
            if fits || full_path.is_none() {
                return target.make(&thumb);
            }
        }
        let full = std::fs::read(full_path.as_ref().unwrap()).map_err(|e| e.to_string())?;
        target.make(&full)
    })
    .await
    .map_err(BizError::msg)?;

    //ETag按生成的图片计算,生成的结果变了(比如参数或编码器变了)ETag也会变
    let etag = format!("\"{:x}\"", md5::compute(&data));
    let size = data.len() as u64;
    let mut raw = if request_util::not_modified(headers, &etag) {
        Raw::new(Vec::new()).status(StatusCode::NOT_MODIFIED)
    } else {
        match request_util::byte_range(headers, size, &etag) {
            ByteRange::Full => Raw::new(data),
            ByteRange::Partial(start, end) => Raw::new(data[start as usize..=end as usize].to_vec())
                .status(StatusCode::PARTIAL_CONTENT)
                .header("content-range", format!("bytes {}-{}/{}", start, end, size)),
            ByteRange::Unsatisfiable => {
                return Ok(Raw::new(Vec::new())
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header("content-range", format!("bytes */{}", size)));
            }
        }
    };
    raw = raw
        .header("content-type", "image/jpeg")
        .header("accept-ranges", "bytes")
        .header("etag", etag)
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
    let thumb_ids: HashSet<i64> = dfs_file_dao::select_thumb_parent_ids(&mut ctx, login.owner_id)
        .await?
        .into_iter()
        .collect();
    let mut files = dfs_file_dao::select_album(&mut ctx, login.owner_id).await?;
    let mut scope = Scope::load(&mut ctx, &login).await?;
    if scope.is_sub() {
        //子账户只显示有浏览权限的文件
        let mut visible = Vec::new();
        for it in files {
            if scope.perms(&mut ctx, &it).await? & perm::VIEW != 0 {
                visible.push(it);
            }
        }
        files = visible;
    }
    let mut list: Vec<(i64, i64, String, bool, i64)> = files
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

/// 检查对文件的权限,没有权限时返回404(下载、缩略图等接口给播放器和图片组件使用,只需要知道不能访问)
async fn check_file(ctx: &mut sqlx_context::DbContext, login: &LoginUser, file: &DfsFile, need: i64) -> Result<(), BizError> {
    if !login.is_sub {
        return Ok(());
    }
    if Scope::load(ctx, login).await?.check(ctx, file, need).await.is_err() {
        return Err(BizError::not_found());
    }
    Ok(())
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

    /// 按路径删除的结果
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DeleteByPath {

        // 移到垃圾箱的文件数量
        pub deleted: usize,

        // 不存在的路径(已经删除或者从来没有过)
        pub not_found: Vec<String>,
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

        // 上传日期(时间戳毫秒,单个文件或文件夹时),上传、覆盖上传、复制时为当时的时间,移动、重命名不变
        pub upload_date: i64,

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
