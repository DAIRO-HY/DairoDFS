use crate::util::media;
use crate::util::media::image_util;
use crate::util::media::image_util::ImageInfo;
use crate::util::shell_util;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::path::Path;

/// 获取RAW图片(CR3,CR2)信息
pub async fn get_info(path: &Path) -> Result<ImageInfo, String> {
    let data = shell_util::exec(&media::exiftool(), [
        "-Orientation".as_ref(), "-ImageWidth".as_ref(), "-ImageHeight".as_ref(), "-CreateDate".as_ref(),
        "-ISO".as_ref(), "-FNumber".as_ref(), "-ExposureTime".as_ref(), "-Make".as_ref(), "-Model".as_ref(), path.as_os_str(),
    ])
    .await?;
    let text = String::from_utf8_lossy(&data);
    let map: HashMap<&str, &str> = text
        .lines()
        .filter_map(|it| it.split_once(':'))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect();
    let mut info = ImageInfo::default();
    if let Some(date) = map.get("Create Date") {
        if let Ok(it) = DateTime::parse_from_str(date, "%Y:%m:%d %H:%M:%S%:z") {
            info.date = it.timestamp_millis();
        } else if let Ok(it) = NaiveDateTime::parse_from_str(date, "%Y:%m:%d %H:%M:%S") {
            info.date = Local.from_local_datetime(&it).earliest().map(|it| it.timestamp_millis()).unwrap_or(0);
        }
    }
    let get = |key: &str| map.get(key).map(|it| it.to_string()).unwrap_or_default();
    info.width = get("Image Width").parse().unwrap_or(0);
    info.height = get("Image Height").parse().unwrap_or(0);
    info.iso = get("ISO").parse().unwrap_or(0);
    info.f_number = get("F Number");
    info.shutter_speed = get("Exposure Time");
    info.make = get("Make");
    info.model = get("Camera Model Name");
    let orientation = get("Orientation");
    info.orientation = if orientation.contains("Rotate 90") {
        6
    } else if orientation.contains("Rotate 180") {
        3
    } else if orientation.contains("Rotate 270") {
        8
    } else {
        1
    };
    Ok(info)
}

/// 从RAW图片中提取内嵌的JPEG预览图,并按照拍摄方向转正
pub async fn to_jpg(path: &Path) -> Result<Vec<u8>, String> {
    let exiftool = media::exiftool();

    //获取内嵌了哪些图片,优先使用最大的那张
    let preview = shell_util::exec(&exiftool, ["-preview:all".as_ref(), path.as_os_str()]).await?;
    let preview = String::from_utf8_lossy(&preview);
    let tag = if preview.contains("Jpg From Raw") {
        Some("-JpgFromRaw")
    } else if preview.contains("Preview Image") {
        Some("-PreviewImage")
    } else if preview.contains("Thumbnail Image") {
        Some("-ThumbnailImage")
    } else {
        None
    };
    let mut jpg = match tag {
        Some(tag) => shell_util::exec(&exiftool, ["-b".as_ref(), tag.as_ref(), path.as_os_str()]).await.unwrap_or_default(),
        None => Vec::new(),
    };
    if jpg.is_empty() {
        //没有内嵌图片时,用libraw解码成TIFF
        let tiff = shell_util::exec(&media::dcraw_emu(), [
            "-T".as_ref(), "-Z".as_ref(), "-".as_ref(), "-mem".as_ref(), "-mmap".as_ref(), path.as_os_str(),
        ])
        .await?;
        jpg = media::blocking(move || image_util::to_jpeg(&tiff, 100)).await?;
    }
    let orientation = get_info(path).await.map(|it| it.orientation).unwrap_or(1);
    if orientation == 1 {
        return Ok(jpg);
    }

    //内嵌的图片没有方向信息,按照RAW图片的拍摄方向转正
    media::blocking(move || {
        let img = image::load_from_memory(&jpg).map_err(|e| e.to_string())?;
        let img = image_util::apply_orientation(img, orientation as u32);
        image_util::encode_jpeg(&img.into_rgb8(), 95)
    })
    .await
}
