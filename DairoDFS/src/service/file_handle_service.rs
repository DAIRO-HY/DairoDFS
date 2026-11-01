use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::{dfs_file_dao, storage_file_dao};
use crate::service::storage_service;
use crate::util::media::video_util::TransferArgument;
use crate::util::media::{image_util, video_util, DliveInfo};
use crate::util::{file_util, id_util, media};
use crate::{db, system_config};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::Notify;

/// 唤醒文件处理任务
static NOTIFY: Notify = Notify::const_new();

/// 视频转码的目标最大边
const VIDEO_TARGET_SIZE: i64 = 1280;

/// 视频转码的目标帧数
const VIDEO_TARGET_FPS: f64 = 30.0;

/// 标记是否正在处理文件(高级页面显示用)
pub static IS_HANDLING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 启动文件处理任务,启动时先处理一遍还没有处理的文件
pub fn start() {
    tokio::spawn(async {
        loop {
            IS_HANDLING.store(true, std::sync::atomic::Ordering::Relaxed);
            handle_all().await;
            IS_HANDLING.store(false, std::sync::atomic::Ordering::Relaxed);

            //没有数据时等待通知,通知会被保留,不会丢失
            NOTIFY.notified().await;
        }
    });
}

/// 通知文件处理任务有新的文件
pub fn notify() {
    NOTIFY.notify_one();
}

/// 处理所有还没有处理的文件
async fn handle_all() {
    if system_config::get().is_read_only {
        //只读模式下不生成附属文件,附属文件从主机同步过来
        return;
    }
    loop {
        let list = match dfs_file_dao::select_no_handle(&mut lib_db::get_context()).await {
            Ok(it) => it,
            Err(e) => {
                eprintln!("-->获取待处理文件失败: {}", e);
                return;
            }
        };
        if list.is_empty() {
            return;
        }
        for file in list {
            let start = Instant::now();
            let (state, msg) = match handle(&file).await {
                Ok(_) => (1, format!("耗时:{:.1}秒", start.elapsed().as_secs_f64())),
                Err(e) => (2, e),
            };
            if let Err(e) = set_state(file.id, state, msg).await {
                //状态写不进去时停止处理,避免一直重复处理同一个文件
                eprintln!("-->设置文件处理状态失败: {}", e);
                return;
            }
        }
    }
}

async fn set_state(id: i64, state: i64, msg: String) -> Result<(), crate::web::biz_error::BizError> {
    let mut ctx = db::begin().await?;
    dfs_file_dao::set_state(&mut ctx, id, state, Some(msg)).await?;
    db::commit(ctx).await
}

/// 处理一个文件:生成属性、缩略图、预览图、预览视频
async fn handle(file: &DfsFile) -> Result<(), String> {
    let storage = storage_file_dao::select_one(&mut lib_db::get_context(), file.storageId)
        .await
        .map_err(|_| format!("本地存储文件:{}不存在", file.storageId))?;
    let path = PathBuf::from(&storage.path);
    if !path.is_file() {
        return Err(format!("文件:{}不存在", storage.path));
    }

    //预览图在生成缩略图和预览图时都要用到,只获取一次
    let mut preview: Option<Option<Vec<u8>>> = None;
    make_property(file, &path).await?;
    make_thumb(file, &path, &mut preview).await?;
    make_preview(file, &path, &mut preview).await?;
    make_video(file, &path).await?;
    Ok(())
}

/// 获取预览图,已经获取过时直接返回
async fn get_preview<'a>(file: &DfsFile, path: &Path, preview: &'a mut Option<Option<Vec<u8>>>) -> Result<&'a Option<Vec<u8>>, String> {
    if preview.is_none() {
        *preview = Some(media::preview_jpg(&file.ext, path).await?);
    }
    Ok(preview.as_ref().unwrap())
}

/// 生成文件属性
async fn make_property(file: &DfsFile, path: &Path) -> Result<(), String> {
    let mut ctx = lib_db::get_context();
    let property = match dfs_file_dao::select_property_by_storage_id(&mut ctx, file.storageId).await.map_err(|e| e.to_string())? {
        //同一个文件已经生成过属性
        Some(it) => Some(it),
        None => media::make_property(&file.ext, path).await?,
    };
    let Some(property) = property else {
        return Ok(());
    };
    let mut ctx = db::begin().await.map_err(|e| e.to_string())?;
    dfs_file_dao::set_property(&mut ctx, file.id, property).await.map_err(|e| e.to_string())?;
    db::commit(ctx).await.map_err(|e| e.to_string())
}

/// 生成缩略图
async fn make_thumb(file: &DfsFile, path: &Path, preview: &mut Option<Option<Vec<u8>>>) -> Result<(), String> {
    if reuse_extra(file, "thumb").await? {
        return Ok(());
    }
    let Some(jpg) = get_preview(file, path, preview).await?.clone() else {
        //不需要生成缩略图的文件
        return Ok(());
    };
    let max_size = system_config::get().thumb_max_size.max(1) as u32;
    let data = media::blocking(move || image_util::resize(&jpg, max_size, 85)).await?;
    add_extra_data(file, "thumb", &data, "image/jpeg").await
}

/// 生成预览图,如PSD,RAW,HEIC等浏览器不能直接显示的图片
async fn make_preview(file: &DfsFile, path: &Path, preview: &mut Option<Option<Vec<u8>>>) -> Result<(), String> {
    if !media::need_preview(&file.ext) {
        return Ok(());
    }
    if reuse_extra(file, "preview").await? {
        return Ok(());
    }
    let Some(jpg) = get_preview(file, path, preview).await?.clone() else {
        return Ok(());
    };

    //压缩图片
    let data = media::blocking(move || image_util::to_jpeg(&jpg, 80)).await?;
    add_extra_data(file, "preview", &data, "image/jpeg").await
}

/// 生成预览视频(转码成兼容性更好、文件更小的视频)
async fn make_video(file: &DfsFile, path: &Path) -> Result<(), String> {
    let is_dlive = file.ext == "dlive";
    if !media::is_video(&file.ext) && !is_dlive {
        return Ok(());
    }
    let extra_name = if is_dlive { "preview-dlive" } else { "preview" };
    if reuse_extra(file, extra_name).await? {
        return Ok(());
    }
    let mut dlive_video_path = None;
    let input = if is_dlive {
        //实况照片先将视频保存到临时文件
        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let temp_path = media::temp_file_path();
        tokio::fs::write(&temp_path, DliveInfo::parse(&data)?.video_data).await.map_err(|e| e.to_string())?;
        dlive_video_path = Some(temp_path.clone());
        temp_path
    } else {
        path.to_path_buf()
    };
    let output = media::temp_file_path();
    let result = transfer_video(file, &input, &output, extra_name).await;
    if let Some(it) = dlive_video_path {
        let _ = tokio::fs::remove_file(it).await;
    }
    let _ = tokio::fs::remove_file(&output).await;
    result
}

/// 转码视频并添加为附属文件
async fn transfer_video(file: &DfsFile, input: &Path, output: &Path, extra_name: &str) -> Result<(), String> {
    let info = video_util::get_info(input).await?;
    let is_horizontal = info.width > info.height;

    //视频最大边
    let max_size = if is_horizontal { info.width } else { info.height };
    if VIDEO_TARGET_SIZE > max_size {
        //视频比目标尺寸小,不需要转码
        return Ok(());
    }
    if VIDEO_TARGET_SIZE == max_size && VIDEO_TARGET_FPS >= info.fps {
        //尺寸一样且帧数不超过目标帧数,不需要转码
        return Ok(());
    }
    let fps = VIDEO_TARGET_FPS.min(info.fps);

    //视频的宽高必须是偶数
    let (width, height) = if is_horizontal {
        let height = (VIDEO_TARGET_SIZE as f64 / info.width as f64 * info.height as f64).round() as i64;
        (VIDEO_TARGET_SIZE, height - height % 2)
    } else {
        let width = (VIDEO_TARGET_SIZE as f64 / info.height as f64 * info.width as f64).round() as i64;
        (width - width % 2, VIDEO_TARGET_SIZE)
    };
    let arg = TransferArgument {
        input,
        time: 0,
        width,
        height,
        fps,
        crf: 22,
        delete_sound: false,
        output,
    };

    //将视频转码成SDR,兼容性更高
    if video_util::is_hdr(input).await {
        video_util::hdr_to_sdr(&arg).await?;
    } else {
        video_util::transfer(&arg).await?;
    }
    let md5 = file_util::md5(output).await.map_err(|e| e.to_string())?;
    let size = tokio::fs::metadata(output).await.map_err(|e| e.to_string())?.len() as i64;
    let storage = storage_service::save(output, &md5).await.map_err(|e| e.to_string())?;
    add_extra(file, extra_name, storage.id, size, "video/mp4").await
}

/// 附属文件已经存在时跳过,同一个文件在其他地方已经生成过附属文件时直接复用
/// - return 是否已经存在或者复用成功
async fn reuse_extra(file: &DfsFile, name: &str) -> Result<bool, String> {
    let mut ctx = lib_db::get_context();
    if dfs_file_dao::select_extra(&mut ctx, file.id, name).await.map_err(|e| e.to_string())?.is_some() {
        return Ok(true);
    }
    let Some(exists) = dfs_file_dao::select_extra_by_storage_id_and_name(&mut ctx, file.storageId, name)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(false);
    };
    add_extra(file, name, exists.storageId, exists.size, exists.contentType.as_deref().unwrap_or_default()).await?;
    Ok(true)
}

/// 保存数据并添加附属文件
async fn add_extra_data(file: &DfsFile, name: &str, data: &[u8], content_type: &str) -> Result<(), String> {
    let storage = storage_service::save_data(data).await.map_err(|e| e.to_string())?;
    add_extra(file, name, storage.id, data.len() as i64, content_type).await
}

/// 添加附属文件
async fn add_extra(file: &DfsFile, name: &str, storage_id: i64, size: i64, content_type: &str) -> Result<(), String> {
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
    dfs_file_dao::insert(&mut ctx, &extra).await.map_err(|e| e.to_string())?;
    db::commit(ctx).await.map_err(|e| e.to_string())
}
