//! 上传文件的处理:生成属性和附属文件(缩略图、预览图、兼容版视频、流畅版视频等)
//!
//! - options:  账号的处理设置(哪些附属文件要生成)
//! - image:    文件属性,缩略图、预览图、视频首帧图
//! - video:    兼容版视频、流畅版视频
//! - compat:   判断视频在手机上能不能直接播放
//! - extra:    附属文件的读取和添加
//! - backfill: 给已有的文件补生成

mod backfill;
mod compat;
mod extra;
mod image;
mod options;
mod video;

pub use backfill::reset_compat_progress;
pub use image::{restore_capture_time, ORIGINAL_DATE, PREVIEW_FRAME_NAME};
pub use options::HandleOptions;

use crate::dao::dfs_file_dao;
use crate::dao::dfs_file_dao::DfsFile;
use crate::util::time_util;
use crate::{db, system_config};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::Notify;

/// 唤醒文件处理任务
static NOTIFY: Notify = Notify::const_new();

/// 标记是否正在处理文件(高级页面显示用)
pub static IS_HANDLING: AtomicBool = AtomicBool::new(false);

/// 启动文件处理任务,启动时先处理一遍还没有处理的文件
pub fn start() {
    tokio::spawn(async {
        loop {
            IS_HANDLING.store(true, Ordering::Relaxed);
            handle_all().await;
            IS_HANDLING.store(false, Ordering::Relaxed);

            //没有数据时等待通知,通知会被保留,不会丢失
            NOTIFY.notified().await;
        }
    });
}

/// 通知文件处理任务有新的文件
pub fn notify() {
    NOTIFY.notify_one();
}

/// 上传的服务器超过这个时间(毫秒)还没有处理时,其他服务器接手处理
const HANDLE_TAKE_OVER_MILLIS: i64 = 10 * 60 * 1000;

/// 处理所有还没有处理的文件
async fn handle_all() {
    if system_config::get().is_read_only {
        //只读模式下不生成附属文件,附属文件从主机同步过来
        return;
    }
    loop {
        //双向同步时只处理自己上传的文件,上传的服务器10分钟还没有处理时,由编号最小的服务器接手
        let (node, is_leader) = crate::service::sync::handle_scope();
        let before = if is_leader { time_util::current_millis() - HANDLE_TAKE_OVER_MILLIS } else { 0 };
        let list = match dfs_file_dao::select_no_handle(&mut lib_db::get_context(), node, before).await {
            Ok(it) => it,
            Err(e) => {
                eprintln!("-->获取待处理文件失败: {}", e);
                return;
            }
        };
        if list.is_empty() {
            //没有新文件时,给已有的文件补生成(每次只处理一批,新上传的文件优先处理)
            //补生成的任务只由一台服务器执行,结果同步到其他服务器
            if is_leader && backfill::run_batch().await {
                continue;
            }
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

/// 处理一个文件:属性总是生成,附属文件按账号的设置生成
async fn handle(file: &DfsFile) -> Result<(), String> {
    let options = HandleOptions::load(file.userId).await?;
    let path = extra::storage_path(file).await?;
    let mut preview = image::PreviewCache::default();
    image::make_property(file, &path).await?;
    image::make_thumb(file, &path, &options, &mut preview).await?;
    image::make_preview(file, &path, &options, &mut preview).await?;
    image::make_preview_frame(file, &path, &options, &mut preview).await?;

    //先生成兼容版:视频不大时,兼容版同时作为流畅版
    video::make_compat(file, &path, &options).await?;
    video::make_smooth(file, &path, &options).await?;
    Ok(())
}
