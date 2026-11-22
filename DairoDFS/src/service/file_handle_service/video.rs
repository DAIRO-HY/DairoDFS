//! 视频的附属文件:兼容版和流畅版
//!
//! | 附属文件名     | 内容                                                       | 账号设置           |
//! |---------------|------------------------------------------------------------|--------------------|
//! | compat        | 手机原生播放不了的视频转成的mp4,分辨率不变                 | 生成兼容版视频     |
//! | preview       | 流畅版:比1280大的视频缩小到1280;不大于1280时就是兼容版     | 生成流畅版视频     |
//! | preview-dlive | 实况照片里的视频(转码后)                                   | 生成流畅版视频     |
//!
//! 先生成兼容版再生成流畅版:原视频不大于1280时不需要缩小,有兼容版时用兼容版作为流畅版,
//! 这样只认识preview的客户端也能播放这些视频

use super::compat::{self, Conversion};
use super::extra;
use super::options::HandleOptions;
use crate::dao::dfs_file_dao::DfsFile;
use crate::util::media::video_util::{self, TransferArgument, VideoCodec, VideoInfo};
use crate::util::media::{self, DliveInfo, TempFile};
use std::future::Future;
use std::path::{Path, PathBuf};

/// 兼容版的附属文件名
pub const COMPAT_NAME: &str = "compat";

/// 流畅版的附属文件名
const SMOOTH_NAME: &str = "preview";

/// 实况照片视频的附属文件名
const DLIVE_SMOOTH_NAME: &str = "preview-dlive";

/// 流畅版的最大边
const SMOOTH_MAX_SIZE: i64 = 1280;

/// 流畅版的最大帧数
const SMOOTH_MAX_FPS: f64 = 30.0;

/// 转码的画质(0-51,越小画质越好,文件越大)
const CRF: i64 = 22;

/// 转码的目标尺寸和帧数
struct Target {

    //宽(偶数)
    width: i64,

    //高(偶数)
    height: i64,

    //帧数,0表示不修改
    fps: f64,
}

/// 生成兼容版:手机原生播放不了的视频转成mp4,分辨率不变
pub async fn make_compat(file: &DfsFile, path: &Path, options: &HandleOptions) -> Result<(), String> {
    if !media::is_video(&file.ext) {
        return Ok(());
    }
    if extra::reuse(file, COMPAT_NAME).await? || !options.compat_video {
        return Ok(());
    }
    let codec = video_util::get_codec(path).await?;
    let output = match compat::conversion(&codec) {
        Conversion::None => return Ok(()),
        Conversion::Remux => remux_or_transcode(path, false, &codec.video, options.video_codec).await?,
        Conversion::ConvertAudio => remux_or_transcode(path, true, &codec.video, options.video_codec).await?,
        Conversion::Transcode => transcode_original_size(path, options.video_codec).await?,
    };
    extra::add_video(file, COMPAT_NAME, output.path()).await
}

/// 不重新编码视频,只换封装;个别文件(如时间戳有问题的AVI)换封装会失败,这时重新编码
async fn remux_or_transcode(path: &Path, convert_audio: bool, video_codec: &str, codec: VideoCodec) -> Result<TempFile, String> {
    let is_hevc = video_codec == "hevc";
    let remuxed = to_temp_file(path, |output| async move { video_util::remux(path, &output, convert_audio, is_hevc).await }).await;
    match remuxed {
        Ok(it) => Ok(it),
        Err(e) => {
            eprintln!("-->视频:{}换封装失败,改为重新编码: {}", path.display(), e);
            transcode_original_size(path, codec).await
        }
    }
}

/// 按原视频的尺寸重新编码
async fn transcode_original_size(path: &Path, codec: VideoCodec) -> Result<TempFile, String> {
    let info = video_util::get_info(path).await?;

    //视频的宽高必须是偶数
    let target = Target {
        width: info.width - info.width % 2,
        height: info.height - info.height % 2,
        fps: 0.0,
    };
    transcode(path, &target, codec).await
}

/// 生成流畅版(实况照片是里面的视频)
pub async fn make_smooth(file: &DfsFile, path: &Path, options: &HandleOptions) -> Result<(), String> {
    let is_dlive = file.ext == media::DLIVE_EXT;
    if !media::is_video(&file.ext) && !is_dlive {
        return Ok(());
    }
    let name = if is_dlive { DLIVE_SMOOTH_NAME } else { SMOOTH_NAME };
    if extra::reuse(file, name).await? || !options.smooth_video {
        return Ok(());
    }

    //实况照片先将视频保存到临时文件
    let dlive_video = if is_dlive { Some(save_dlive_video(path).await?) } else { None };
    let input = dlive_video.as_ref().map_or(path, |it| it.path());
    let info = video_util::get_info(input).await?;
    let Some(target) = smooth_target(&info) else {
        //视频不大,不需要缩小;有兼容版时用兼容版作为流畅版(同一个存储文件,不占空间)
        if let Some(compat) = extra::get(file, COMPAT_NAME).await? {
            extra::add(file, name, compat.storageId, compat.size, "video/mp4").await?;
        }
        return Ok(());
    };
    let output = transcode(input, &target, options.video_codec).await?;
    extra::add_video(file, name, output.path()).await
}

/// 实况照片里的视频保存到临时文件
async fn save_dlive_video(path: &Path) -> Result<TempFile, String> {
    let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let video_data = DliveInfo::parse(&data)?.video_data;
    let temp = TempFile::new(video_data.len() as u64);
    tokio::fs::write(temp.path(), video_data).await.map_err(|e| e.to_string())?;
    Ok(temp)
}

/// 流畅版的尺寸和帧数,视频本来就不大时返回None(不需要转码)
fn smooth_target(info: &VideoInfo) -> Option<Target> {
    let is_horizontal = info.width > info.height;

    //视频最大边
    let max_size = if is_horizontal { info.width } else { info.height };
    if SMOOTH_MAX_SIZE > max_size {
        return None;
    }
    if SMOOTH_MAX_SIZE == max_size && SMOOTH_MAX_FPS >= info.fps {
        //尺寸一样且帧数不超过目标帧数
        return None;
    }

    //视频的宽高必须是偶数
    let (width, height) = if is_horizontal {
        let height = (SMOOTH_MAX_SIZE as f64 / info.width as f64 * info.height as f64).round() as i64;
        (SMOOTH_MAX_SIZE, height - height % 2)
    } else {
        let width = (SMOOTH_MAX_SIZE as f64 / info.height as f64 * info.width as f64).round() as i64;
        (width - width % 2, SMOOTH_MAX_SIZE)
    };
    Some(Target {
        width,
        height,
        fps: SMOOTH_MAX_FPS.min(info.fps),
    })
}

/// 转码到临时文件,HDR视频同时转成SDR(兼容性更高)
async fn transcode(input: &Path, target: &Target, codec: VideoCodec) -> Result<TempFile, String> {
    let is_hdr = video_util::is_hdr(input).await;
    to_temp_file(input, |output| async move {
        let arg = TransferArgument {
            input,
            time: 0,
            width: target.width,
            height: target.height,
            fps: target.fps,
            crf: CRF,
            delete_sound: false,
            codec,
            output: &output,
        };
        if is_hdr {
            video_util::hdr_to_sdr(&arg).await
        } else {
            video_util::transfer(&arg).await
        }
    })
    .await
}

/// 将视频输出到临时文件
/// 输出的视频一般比原视频小,按原视频的大小在RAM硬盘上预留空间;输出的比预计的大、RAM硬盘空间不够时,放在硬盘上重新输出
async fn to_temp_file<F, Fut>(input: &Path, run: F) -> Result<TempFile, String>
where
    F: Fn(PathBuf) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let input_size = tokio::fs::metadata(input).await.map_err(|e| e.to_string())?.len();
    let output = TempFile::new(input_size);
    match run(output.path().to_path_buf()).await {
        Ok(()) => Ok(output),
        Err(_) if output.is_ram_disk_full() => {
            drop(output);
            let output = TempFile::disk();
            run(output.path().to_path_buf()).await?;
            Ok(output)
        }
        Err(e) => Err(e),
    }
}
