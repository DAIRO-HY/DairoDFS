//! 给已有的文件补生成附属文件或属性(没有新上传的文件时分批执行,新上传的文件优先处理)
//!
//! | 任务           | 进度                                   | 什么时候执行                         |
//! |---------------|----------------------------------------|--------------------------------------|
//! | 补首帧图       | 内存里,每次启动从头补一遍(失败的会重试) | 总是执行,只补设置了生成缩略图的账号   |
//! | 补镜头和焦距   | data/.property_lens_after_id           | 每个数据目录只补一遍                 |
//! | 补转兼容版视频 | data/.compat_after_id                  | 系统设置里开启时,只补设置了兼容版的账号 |

use super::options::HandleOptions;
use super::{extra, image, video};
use crate::dao::dfs_file_dao;
use crate::dao::dfs_file_dao::DfsFile;
use crate::util::media;
use crate::{db, system_config};
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};

/// 补生成视频首帧图时,已经处理到的文件ID(每次启动后从头补一遍,处理失败的文件下次启动再重试)
static PREVIEW_FRAME_AFTER_ID: AtomicI64 = AtomicI64::new(0);

/// 执行一批补生成的任务
/// - return 是否还有需要处理的文件
pub async fn run_batch() -> bool {
    make_preview_frame_todo().await || make_lens_todo().await || make_compat_todo().await
}

/// 进度文件(记录已经处理到的文件ID)
fn progress_path(name: &str) -> PathBuf {
    PathBuf::from(&crate::application::ARGS.data_dir).join(name)
}

/// 补充镜头和焦距信息的进度文件,老版本生成的属性里没有这两项
const LENS_PROGRESS: &str = ".property_lens_after_id";

/// 补转兼容版视频的进度文件
const COMPAT_PROGRESS: &str = ".compat_after_id";

/// 读取进度,全部完成时返回None
fn read_progress(name: &str) -> Option<i64> {
    let after_id: i64 = std::fs::read_to_string(progress_path(name)).ok().and_then(|it| it.trim().parse().ok()).unwrap_or(0);
    (after_id != i64::MAX).then_some(after_id)
}

/// 保存进度,保存失败时返回false(停止补生成,避免一直重复处理同一批文件)
fn save_progress(name: &str, after_id: i64) -> bool {
    if let Err(e) = std::fs::write(progress_path(name), after_id.to_string()) {
        eprintln!("-->保存进度:{}失败: {}", name, e);
        return false;
    }
    true
}

/// 补转兼容版视频的进度清零(系统设置里重新开启时从头补一遍)
pub fn reset_compat_progress() {
    let _ = std::fs::remove_file(progress_path(COMPAT_PROGRESS));
}

/// 给已有的视频补生成首帧图(一批)
async fn make_preview_frame_todo() -> bool {
    let after_id = PREVIEW_FRAME_AFTER_ID.load(Ordering::Relaxed);
    let list = match dfs_file_dao::select_preview_frame_todo(&mut lib_db::get_context(), after_id, video_exts()).await {
        Ok(it) => it,
        Err(e) => {
            eprintln!("-->获取需要补生成首帧图的视频失败: {}", e);
            return false;
        }
    };
    let Some(last) = list.last() else {
        return false;
    };
    PREVIEW_FRAME_AFTER_ID.store(last.id, Ordering::Relaxed);
    for file in list {
        let result = async {
            let options = HandleOptions::load(file.userId).await?;
            let path = extra::storage_path(&file).await?;
            image::make_preview_frame(&file, &path, &options, &mut Default::default()).await
        };
        if let Err(e) = result.await {
            //补生成失败不影响文件状态,下次启动时再重试
            eprintln!("-->文件:{}补生成首帧图失败: {}", file.id, e);
        }
    }
    true
}

/// 给已有的照片补充镜头和焦距信息(一批),只添加这几项,属性里的其他内容(包括修改过的拍摄时间)不变
async fn make_lens_todo() -> bool {
    let Some(after_id) = read_progress(LENS_PROGRESS) else {
        return false;
    };
    let exts = media::exif_exts().into_iter().map(String::from).collect();
    let list = match dfs_file_dao::select_lens_todo(&mut lib_db::get_context(), after_id, exts).await {
        Ok(it) => it,
        Err(e) => {
            eprintln!("-->获取需要补充镜头信息的文件失败: {}", e);
            return false;
        }
    };
    let Some(last) = list.last() else {
        //全部处理完成,以后不再处理
        save_progress(LENS_PROGRESS, i64::MAX);
        return false;
    };
    let last_id = last.id;
    for file in list {
        if let Err(e) = add_lens(&file).await {
            eprintln!("-->文件:{}补充镜头信息失败: {}", file.id, e);
        }
    }
    save_progress(LENS_PROGRESS, last_id)
}

/// 给一个文件的属性补充镜头和焦距信息
async fn add_lens(file: &DfsFile) -> Result<(), String> {
    let Some(property) = &file.property else {
        return Ok(());
    };
    let Ok(serde_json::Value::Object(mut map)) = serde_json::from_str::<serde_json::Value>(property) else {
        return Ok(());
    };
    let path = extra::storage_path(file).await?;
    let Some(new_property) = media::make_property(&file.ext, &path).await? else {
        return Ok(());
    };
    let Ok(serde_json::Value::Object(new_map)) = serde_json::from_str::<serde_json::Value>(&new_property) else {
        return Ok(());
    };
    let mut changed = false;
    for key in ["lens", "focalLength", "focalLength35"] {
        if let Some(value) = new_map.get(key) {
            if !map.contains_key(key) {
                map.insert(key.to_string(), value.clone());
                changed = true;
            }
        }
    }
    if !changed {
        return Ok(());
    }
    let mut ctx = db::begin().await.map_err(|e| e.to_string())?;
    dfs_file_dao::set_property(&mut ctx, file.id, serde_json::Value::Object(map).to_string()).await.map_err(|e| e.to_string())?;
    db::commit(ctx).await.map_err(|e| e.to_string())
}

/// 给已有的视频补转兼容版(一批),系统设置里开启时才执行
async fn make_compat_todo() -> bool {
    if !system_config::get().compat_backfill {
        return false;
    }
    let Some(after_id) = read_progress(COMPAT_PROGRESS) else {
        return false;
    };
    let list = match dfs_file_dao::select_compat_todo(&mut lib_db::get_context(), after_id, video_exts()).await {
        Ok(it) => it,
        Err(e) => {
            eprintln!("-->获取需要补转兼容版的视频失败: {}", e);
            return false;
        }
    };
    let Some(last) = list.last() else {
        //全部处理完成,以后不再处理(系统设置里重新开启时从头补一遍)
        save_progress(COMPAT_PROGRESS, i64::MAX);
        return false;
    };
    let last_id = last.id;
    for file in list {
        if let Err(e) = make_compat(&file).await {
            //失败的不重试,可以在高级页面重新处理
            eprintln!("-->文件:{}补转兼容版失败: {}", file.id, e);
        }
    }
    save_progress(COMPAT_PROGRESS, last_id)
}

/// 给一个视频补转兼容版
/// 以前不认识的视频格式(如wmv)当时没有生成属性和流畅版,这里一起补上
async fn make_compat(file: &DfsFile) -> Result<(), String> {
    let options = HandleOptions::load(file.userId).await?;
    let path = extra::storage_path(file).await?;
    image::make_property(file, &path).await?;
    video::make_compat(file, &path, &options).await?;
    video::make_smooth(file, &path, &options).await
}

/// 视频的扩展名
fn video_exts() -> Vec<String> {
    media::video_exts().into_iter().map(String::from).collect()
}
