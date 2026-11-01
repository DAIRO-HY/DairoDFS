use crate::util::media;
use crate::util::shell_util;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// 视频信息(字段名与老版本一致,保存在dfs_file.property里)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VideoInfo {

    //宽
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub width: i64,

    //高
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub height: i64,

    //帧数
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub fps: f64,

    //视频比特率
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub bitrate: i64,

    //视频时长(毫秒)
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub duration: i64,

    //视频创建时间戳(毫秒)
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub date: i64,
}

fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

/// 转码参数
#[derive(Debug)]
pub struct TransferArgument<'a> {

    //输入文件
    pub input: &'a Path,

    //截取时长(秒),0表示不截取
    pub time: i64,

    //目标宽(必须是偶数)
    pub width: i64,

    //目标高(必须是偶数)
    pub height: i64,

    //目标帧率,0表示不修改
    pub fps: f64,

    //视频画质，0-51 0：无损（文件大） 51：最低画质（文件小）
    pub crf: i64,

    //删除声音
    pub delete_sound: bool,

    //输出文件
    pub output: &'a Path,
}

/// 获取视频信息
pub async fn get_info(path: &Path) -> Result<VideoInfo, String> {
    let data = shell_util::exec(&media::ffprobe(), [
        "-v".as_ref(), "error".as_ref(), "-print_format".as_ref(), "json".as_ref(), "-show_streams".as_ref(), path.as_os_str(),
    ])
    .await?;
    let json: Value = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let Some(stream) = json["streams"]
        .as_array()
        .and_then(|it| it.iter().find(|s| s["codec_type"] == "video"))
    else {
        return Err("没有获取到视频信息".to_string());
    };
    let mut info = VideoInfo {
        width: stream["width"].as_i64().unwrap_or(0),
        height: stream["height"].as_i64().unwrap_or(0),
        bitrate: stream["bit_rate"].as_str().and_then(|it| it.parse().ok()).unwrap_or(0),
        ..Default::default()
    };
    if let Some(duration) = stream["duration"].as_str().and_then(|it| it.parse::<f64>().ok()) {
        info.duration = (duration * 1000.0) as i64;
    }
    if let Some((num, den)) = stream["r_frame_rate"].as_str().and_then(|it| it.split_once('/')) {
        let (num, den) = (num.parse::<f64>().unwrap_or(0.0), den.parse::<f64>().unwrap_or(0.0));
        if den > 0.0 {
            info.fps = (num / den * 100.0).round() / 100.0;
        }
    }
    if let Some(time) = stream["tags"]["creation_time"].as_str() {
        //格式: 2024-07-15T07:22:24.000000Z
        let time = time.replace('T', " ");
        let time = time.split('.').next().unwrap_or_default();
        if let Ok(it) = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S") {
            info.date = it.and_utc().timestamp_millis();
        }
    }

    //竖着拍摄的视频,宽高需要调换
    let rotation = stream["side_data_list"]
        .as_array()
        .and_then(|it| it.iter().find_map(|s| s["rotation"].as_i64()))
        .or_else(|| stream["tags"]["rotate"].as_str().and_then(|it| it.parse().ok()))
        .unwrap_or(0);
    if rotation.abs() == 90 || rotation.abs() == 270 {
        std::mem::swap(&mut info.width, &mut info.height);
    }
    Ok(info)
}

/// 判断视频是否HDR
pub async fn is_hdr(path: &Path) -> bool {
    let Ok(data) = shell_util::exec(&media::ffprobe(), [
        "-v".as_ref(), "error".as_ref(), "-select_streams".as_ref(), "v:0".as_ref(),
        "-show_entries".as_ref(), "stream=color_transfer".as_ref(), "-of".as_ref(), "json".as_ref(), path.as_os_str(),
    ])
    .await
    else {
        return false;
    };
    let text = String::from_utf8_lossy(&data);
    text.contains("arib-std-b67") || text.contains("smpte2084")
}

/// 获取视频第一帧图片
pub async fn to_jpg(path: &Path) -> Result<Vec<u8>, String> {
    if !is_hdr(path).await {
        return first_frame(path).await;
    }

    //先将HDR转SDR再截图，避免取出来的图片泛白
    let temp_path = media::temp_file_path();
    let arg = TransferArgument {
        input: path,
        time: 1,
        width: 0,
        height: 0,
        fps: 1.0,
        crf: 18,
        delete_sound: true,
        output: &temp_path,
    };
    let result = match hdr_to_sdr(&arg).await {
        Ok(_) => first_frame(&temp_path).await,
        Err(e) => Err(e),
    };
    let _ = tokio::fs::remove_file(&temp_path).await;
    result
}

/// 截取视频第一帧
async fn first_frame(path: &Path) -> Result<Vec<u8>, String> {
    shell_util::exec(&media::ffmpeg(), [
        "-v".as_ref(), "error".as_ref(), "-i".as_ref(), path.as_os_str(), "-frames:v".as_ref(), "1".as_ref(),
        "-q:v".as_ref(), "1".as_ref(), "-f".as_ref(), "image2pipe".as_ref(), "-vcodec".as_ref(), "mjpeg".as_ref(), "-".as_ref(),
    ])
    .await
}

/// 视频转码(SDR转SDR,HDR转HDR),编码为H.265
pub async fn transfer(arg: &TransferArgument<'_>) -> Result<(), String> {
    let scale = format!("scale=w={}:h={}", arg.width, arg.height);
    run_transfer(arg, &scale).await
}

/// HDR视频转SDR视频,编码为H.265
pub async fn hdr_to_sdr(arg: &TransferArgument<'_>) -> Result<(), String> {
    let size = if arg.width > 0 && arg.height > 0 {
        format!("w={}:h={}:", arg.width, arg.height)
    } else {
        String::new()
    };
    let filter = format!(
        "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale={}t=bt709:m=bt709:r=tv,format=yuv420p",
        size
    );
    run_transfer(arg, &filter).await
}

async fn run_transfer(arg: &TransferArgument<'_>, filter: &str) -> Result<(), String> {
    let mut args: Vec<String> = vec!["-v".into(), "error".into(), "-i".into(), arg.input.to_string_lossy().into()];
    if arg.time > 0 {
        args.extend(["-t".into(), arg.time.to_string()]);
    }
    if arg.width > 0 || filter.starts_with("zscale") {
        args.extend(["-vf".into(), filter.to_string()]);
    }

    //-tag:v hvc1兼容苹果设备,ffmpeg从H.264转H.265时,可能转-tag:v hev1得编码,导致苹果设备上无法播放
    args.extend([
        "-c:v".into(), "libx265".into(), "-tag:v".into(), "hvc1".into(),
        "-crf".into(), arg.crf.to_string(), "-preset".into(), "medium".into(),
    ]);
    if arg.fps > 0.0 {
        args.extend(["-r".into(), format!("{:.2}", arg.fps)]);
    }
    if arg.delete_sound {
        args.push("-an".into());
    }
    args.extend(["-f".into(), "mp4".into(), "-y".into(), arg.output.to_string_lossy().into()]);
    shell_util::exec(&media::ffmpeg(), &args).await?;
    Ok(())
}
