//! 判断视频在手机上能不能直接播放,不能播放时怎么转换
//!
//! iOS(AVPlayer)和安卓(系统播放器)都能直接播放的视频:
//! - 封装格式:mp4/mov/m4v/3gp(ffprobe识别为同一种格式)
//! - 视频编码:H.264(8位4:2:0)、H.265(8位或10位4:2:0)
//! - 音频编码:AAC、MP3,或者没有声音

use crate::util::media::video_util::CodecInfo;

/// 手机不能直接播放时的转换方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conversion {

    /// 能直接播放,不需要转换
    None,

    /// 只换成mp4封装,视频和音频都直接复制(几秒就能完成,画质不变)
    Remux,

    /// 换成mp4封装,视频直接复制,音频转成AAC
    ConvertAudio,

    /// 视频需要重新编码
    Transcode,
}

/// 判断视频需要怎么转换
pub fn conversion(info: &CodecInfo) -> Conversion {
    if !is_video_supported(&info.video, &info.pix_fmt) {
        return Conversion::Transcode;
    }
    if !info.audio.as_deref().is_none_or(is_audio_supported) {
        return Conversion::ConvertAudio;
    }
    if !is_format_supported(&info.format) {
        return Conversion::Remux;
    }
    Conversion::None
}

/// 封装格式是否支持(ffprobe把mp4/mov/m4v/3gp都识别为"mov,mp4,m4a,3gp,3g2,mj2")
fn is_format_supported(format: &str) -> bool {
    format.split(',').any(|it| it == "mp4")
}

/// 视频编码和像素格式是否支持(手机只能硬件解码4:2:0的视频,H.264不支持10位)
fn is_video_supported(codec: &str, pix_fmt: &str) -> bool {
    match codec {
        "h264" => matches!(pix_fmt, "yuv420p" | "yuvj420p"),
        "hevc" => matches!(pix_fmt, "yuv420p" | "yuvj420p" | "yuv420p10le"),
        _ => false,
    }
}

/// 音频编码是否支持
fn is_audio_supported(codec: &str) -> bool {
    matches!(codec, "aac" | "mp3")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(format: &str, video: &str, pix_fmt: &str, audio: Option<&str>) -> CodecInfo {
        CodecInfo {
            format: format.to_string(),
            video: video.to_string(),
            pix_fmt: pix_fmt.to_string(),
            audio: audio.map(String::from),
        }
    }

    const MP4: &str = "mov,mp4,m4a,3gp,3g2,mj2";

    #[test]
    fn test_conversion() {
        assert_eq!(conversion(&info(MP4, "h264", "yuv420p", Some("aac"))), Conversion::None);
        assert_eq!(conversion(&info(MP4, "hevc", "yuv420p10le", None)), Conversion::None);
        assert_eq!(conversion(&info("matroska,webm", "h264", "yuv420p", Some("aac"))), Conversion::Remux);
        assert_eq!(conversion(&info("flv", "h264", "yuv420p", Some("mp3"))), Conversion::Remux);
        assert_eq!(conversion(&info("matroska,webm", "h264", "yuv420p", Some("ac3"))), Conversion::ConvertAudio);
        assert_eq!(conversion(&info(MP4, "h264", "yuv420p", Some("pcm_s16le"))), Conversion::ConvertAudio);
        assert_eq!(conversion(&info("asf", "wmv3", "yuv420p", Some("wmav2"))), Conversion::Transcode);
        assert_eq!(conversion(&info("avi", "mpeg4", "yuv420p", Some("mp3"))), Conversion::Transcode);
        assert_eq!(conversion(&info(MP4, "h264", "yuv420p10le", Some("aac"))), Conversion::Transcode);
        assert_eq!(conversion(&info(MP4, "prores", "yuv422p10le", Some("aac"))), Conversion::Transcode);
    }
}
