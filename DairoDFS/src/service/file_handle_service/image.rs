//! 文件属性和图片类的附属文件:缩略图、预览图、视频首帧图

use super::extra;
use super::options::HandleOptions;
use crate::dao::dfs_file_dao;
use crate::dao::dfs_file_dao::DfsFile;
use crate::util::media::{self, image_util};
use crate::{db, system_config};
use std::path::Path;

/// 缩略图的附属文件名
const THUMB_NAME: &str = "thumb";

/// 图片预览图的附属文件名
const PREVIEW_NAME: &str = "preview";

/// 视频首帧图(和视频一样的尺寸)的附属文件名,在线生成各种尺寸的缩略图时使用
pub const PREVIEW_FRAME_NAME: &str = "preview-frame";

/// 视频首帧图的JPEG质量
const PREVIEW_FRAME_QUALITY: u8 = 85;

/// 修改拍摄时间时,原来的拍摄时间保存在这个属性里,恢复时使用
pub const ORIGINAL_DATE: &str = "originalDate";

/// 从原文件取出的预览图(JPEG),生成缩略图、预览图、首帧图时都要用到,只取一次
/// 用到时才去取,设置了不生成缩略图时不会取
#[derive(Default)]
pub struct PreviewCache(Option<Option<Vec<u8>>>);

impl PreviewCache {

    /// 获取预览图,不能生成预览图的文件返回None
    async fn get(&mut self, file: &DfsFile, path: &Path) -> Result<Option<Vec<u8>>, String> {
        if self.0.is_none() {
            self.0 = Some(media::preview_jpg(&file.ext, path).await?);
        }
        Ok(self.0.clone().unwrap())
    }
}

/// 生成文件属性(总是生成,不受账号设置影响)
pub async fn make_property(file: &DfsFile, path: &Path) -> Result<(), String> {
    if file.property.is_some() {
        //已经有属性(比如复制的文件带上了源文件的属性,包括修改过的拍摄时间),不重新生成
        return Ok(());
    }
    let mut ctx = lib_db::get_context();
    let property = match dfs_file_dao::select_property_by_storage_id(&mut ctx, file.storageId).await.map_err(|e| e.to_string())? {
        //同一个文件已经生成过属性,其他文件修改过的拍摄时间不能带过来
        Some(it) => Some(restore_capture_time(&it)),
        None => media::make_property(&file.ext, path).await?,
    };
    let Some(property) = property else {
        return Ok(());
    };
    let mut ctx = db::begin().await.map_err(|e| e.to_string())?;
    dfs_file_dao::set_property(&mut ctx, file.id, property).await.map_err(|e| e.to_string())?;
    db::commit(ctx).await.map_err(|e| e.to_string())
}

/// 去掉修改过的拍摄时间,恢复成照片本身的拍摄时间
pub fn restore_capture_time(property: &str) -> String {
    let Ok(serde_json::Value::Object(mut map)) = serde_json::from_str::<serde_json::Value>(property) else {
        return property.to_string();
    };
    let Some(original) = map.remove(ORIGINAL_DATE) else {
        return property.to_string();
    };
    if original.as_i64().is_some_and(|it| it > 0) {
        map.insert("date".to_string(), original);
    } else {
        map.remove("date");
    }
    serde_json::Value::Object(map).to_string()
}

/// 生成缩略图
pub async fn make_thumb(file: &DfsFile, path: &Path, options: &HandleOptions, preview: &mut PreviewCache) -> Result<(), String> {
    if extra::reuse(file, THUMB_NAME).await? || !options.thumb {
        return Ok(());
    }
    let Some(jpg) = preview.get(file, path).await? else {
        //不需要生成缩略图的文件
        return Ok(());
    };
    let max_size = system_config::get().thumb_max_size.max(1) as u32;
    let data = media::blocking(move || image_util::resize(&jpg, max_size, 85)).await?;
    extra::add_data(file, THUMB_NAME, &data, "image/jpeg").await
}

/// 生成预览图,如PSD,RAW,HEIC等浏览器不能直接显示的图片
pub async fn make_preview(file: &DfsFile, path: &Path, options: &HandleOptions, preview: &mut PreviewCache) -> Result<(), String> {
    if !media::need_preview(&file.ext) {
        return Ok(());
    }
    if extra::reuse(file, PREVIEW_NAME).await? || !options.thumb {
        return Ok(());
    }
    let Some(jpg) = preview.get(file, path).await? else {
        return Ok(());
    };

    //压缩图片
    let data = media::blocking(move || image_util::to_jpeg(&jpg, 80)).await?;
    extra::add_data(file, PREVIEW_NAME, &data, "image/jpeg").await
}

/// 生成视频首帧图(和视频一样的尺寸),在线生成各种尺寸的缩略图时使用,不用每次都从视频里截取
pub async fn make_preview_frame(file: &DfsFile, path: &Path, options: &HandleOptions, preview: &mut PreviewCache) -> Result<(), String> {
    if !media::is_video(&file.ext) {
        return Ok(());
    }
    if extra::reuse(file, PREVIEW_FRAME_NAME).await? || !options.thumb {
        return Ok(());
    }
    let Some(jpg) = preview.get(file, path).await? else {
        return Ok(());
    };

    //截取的图片质量很高(文件很大),重新压缩
    let data = media::blocking(move || image_util::to_jpeg(&jpg, PREVIEW_FRAME_QUALITY)).await?;
    extra::add_data(file, PREVIEW_FRAME_NAME, &data, "image/jpeg").await
}
