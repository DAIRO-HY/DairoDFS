pub mod image_util;
pub mod raw_util;
pub mod video_util;

use crate::application;
use crate::util::shell_util;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// 可以直接解码的图片
const IMAGE_EXTS: [&str; 10] = ["jpg", "jpeg", "jfif", "png", "bmp", "gif", "ico", "tiff", "webp", "tga"];

/// 需要借助ImageMagick解码的图片
const MAGICK_IMAGE_EXTS: [&str; 5] = ["svg", "wmf", "wmz", "jp2", "eps"];

/// PhotoShop文件
const PSD_EXTS: [&str; 3] = ["psd", "psb", "ai"];

/// 专业相机RAW图片
const RAW_EXTS: [&str; 2] = ["cr3", "cr2"];

/// HEIF格式图片(iPhone拍摄的照片,相机拍摄的HIF照片)
const HEIF_EXTS: [&str; 2] = ["heic", "hif"];

/// 实况照片
const DLIVE_EXT: &str = "dlive";

/// 视频
const VIDEO_EXTS: [&str; 8] = ["mp4", "mov", "avi", "mkv", "flv", "rm", "rmvb", "3gp"];

/// ffmpeg程序路径
pub fn ffmpeg() -> PathBuf {
    shell_util::tool_path(&application::FFMPEG_PATH, "ffmpeg")
}

/// ffprobe程序路径
pub fn ffprobe() -> PathBuf {
    shell_util::tool_path(&application::FFPROBE_PATH, "ffprobe")
}

/// libraw的dcraw模拟器程序路径(Linux下编译安装到了系统目录)
pub fn dcraw_emu() -> PathBuf {
    shell_util::tool_path(&application::LIBRAW_PATH.join("LibRaw-0.21.2/bin"), "dcraw_emu")
}

/// exiftool程序路径(Windows下使用安装目录下的程序,其他系统安装到了系统目录)
pub fn exiftool() -> PathBuf {
    if cfg!(windows) {
        //安装目录下的子目录名带版本号,如 exiftool-13.59_64
        let folder = std::fs::read_dir(application::EXIFTOOL_PATH.as_path())
            .into_iter()
            .flatten()
            .flatten()
            .map(|it| it.path())
            .find(|it| it.join("exiftool.exe").is_file());
        return match folder {
            Some(it) => it.join("exiftool.exe"),
            None => PathBuf::from("exiftool"),
        };
    }
    shell_util::brew_tool_path("exiftool")
}

/// ImageMagick程序路径
pub fn magick() -> PathBuf {
    shell_util::brew_tool_path("magick")
}

/// 是否视频文件
pub fn is_video(ext: &str) -> bool {
    VIDEO_EXTS.contains(&ext)
}

/// 是否需要生成预览图(浏览器不能直接显示的图片)
pub fn need_preview(ext: &str) -> bool {
    PSD_EXTS.contains(&ext) || RAW_EXTS.contains(&ext) || HEIF_EXTS.contains(&ext) || ext == DLIVE_EXT
}

/// 生成文件属性(JSON),不需要生成属性的文件返回None
pub async fn make_property(ext: &str, path: &Path) -> Result<Option<String>, String> {
    let json = if IMAGE_EXTS.contains(&ext) {
        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let info = blocking(move || image_util::get_info(&data)).await?;
        serde_json::to_string(&info)
    } else if RAW_EXTS.contains(&ext) {
        serde_json::to_string(&raw_util::get_info(path).await?)
    } else if is_video(ext) {
        serde_json::to_string(&video_util::get_info(path).await?)
    } else if MAGICK_IMAGE_EXTS.contains(&ext) || PSD_EXTS.contains(&ext) || HEIF_EXTS.contains(&ext) || ext == DLIVE_EXT {
        //先转换成JPEG再获取信息
        let Some(jpg) = preview_jpg(ext, path).await? else {
            return Ok(None);
        };
        let info = blocking(move || image_util::get_info(&jpg)).await?;
        serde_json::to_string(&info)
    } else {
        return Ok(None);
    };
    Ok(Some(json.map_err(|e| e.to_string())?))
}

/// 获取文件的预览图(JPEG),用来生成缩略图和预览图,不支持的文件返回None
pub async fn preview_jpg(ext: &str, path: &Path) -> Result<Option<Vec<u8>>, String> {
    let data = if ext == "jpg" || ext == "jpeg" || ext == "jfif" {
        tokio::fs::read(path).await.map_err(|e| e.to_string())?
    } else if IMAGE_EXTS.contains(&ext) {
        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        blocking(move || image_util::to_jpeg(&data, 100)).await?
    } else if MAGICK_IMAGE_EXTS.contains(&ext) || HEIF_EXTS.contains(&ext) {
        magick_to_jpg(path.as_os_str().to_string_lossy().as_ref(), None).await?
    } else if PSD_EXTS.contains(&ext) {
        shell_util::exec(&ffmpeg(), [
            "-v".as_ref(), "error".as_ref(), "-i".as_ref(), path.as_os_str(), "-frames:v".as_ref(), "1".as_ref(),
            "-q:v".as_ref(), "2".as_ref(), "-f".as_ref(), "image2".as_ref(), "-vcodec".as_ref(), "mjpeg".as_ref(), "-".as_ref(),
        ])
        .await?
    } else if RAW_EXTS.contains(&ext) {
        raw_util::to_jpg(path).await?
    } else if is_video(ext) {
        video_util::to_jpg(path).await?
    } else if ext == DLIVE_EXT {
        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let dlive = DliveInfo::parse(&data)?;
        match dlive.photo_ext.as_str() {
            "jpg" | "jpeg" => dlive.photo_data.to_vec(),
            "heic" | "hif" => magick_to_jpg("-", Some(dlive.photo_data.to_vec())).await?,
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    };
    Ok(Some(data))
}

/// 用ImageMagick转换成JPEG,并按照拍摄方向转正
/// - input 文件路径,为"-"时从data读取
async fn magick_to_jpg(input: &str, data: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
    shell_util::exec_with_input(&magick(), [input, "-auto-orient", "-quality", "100", "JPEG:-"], data).await
}

/// 实况照片信息
/// 文件格式: 头部(以减号结尾,内容为 x|照片格式|照片大小|视频格式) + 照片数据 + 视频数据
pub struct DliveInfo<'a> {

    //照片格式
    pub photo_ext: String,

    //照片数据
    pub photo_data: &'a [u8],

    //视频数据
    pub video_data: &'a [u8],
}

impl<'a> DliveInfo<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, String> {
        let error = || "实况照片格式不正确".to_string();
        let head_end = data.iter().position(|it| *it == b'-').ok_or_else(error)?;
        let head = String::from_utf8_lossy(&data[..head_end]).to_string();
        let head: Vec<&str> = head.split('|').collect();
        if head.len() < 4 {
            return Err(error());
        }
        let photo_size: usize = head[2].parse().map_err(|_| error())?;
        let photo_end = head_end + 1 + photo_size;
        if photo_end > data.len() {
            return Err(error());
        }
        Ok(Self {
            photo_ext: head[1].to_lowercase(),
            photo_data: &data[head_end + 1..photo_end],
            video_data: &data[photo_end..],
        })
    }
}

/// 生成一个临时文件路径
pub fn temp_file_path() -> PathBuf {
    static NO: AtomicU64 = AtomicU64::new(0);
    let no = NO.fetch_add(1, Ordering::Relaxed);
    application::TEMP_PATH.join(format!("media-{}-{}", crate::util::time_util::current_micros(), no))
}

/// 在阻塞线程里执行耗CPU的图片处理
pub async fn blocking<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| e.to_string())?
}
