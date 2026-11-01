use chrono::{Local, NaiveDateTime, TimeZone};
use exif::{In, Tag, Value};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageReader, RgbImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// 照片信息(字段名与老版本一致,保存在dfs_file.property里)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImageInfo {

    //宽
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub width: i64,

    //高
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub height: i64,

    //拍摄时间(时间戳毫秒)
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub date: i64,

    //相机制造商
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub make: String,

    //设备型号
    #[serde(default, rename = "camera", skip_serializing_if = "String::is_empty")]
    pub model: String,

    //光圈大小
    #[serde(default, rename = "fNumber", skip_serializing_if = "String::is_empty")]
    pub f_number: String,

    //快门速度
    #[serde(default, rename = "shutterSpeed", skip_serializing_if = "String::is_empty")]
    pub shutter_speed: String,

    //曝光
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub iso: i64,

    //纬度
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub lat: f64,

    //经度
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub long: f64,

    //图片方向 1:正常 3:旋转180度 6:需要顺时针旋转90度 8:需要逆时针旋转90度
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub orientation: i64,
}

fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

/// 获取图片信息(尺寸和EXIF)
pub fn get_info(data: &[u8]) -> Result<ImageInfo, String> {
    let (width, height) = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| e.to_string())?;
    let mut info = ImageInfo {
        width: width as i64,
        height: height as i64,
        ..Default::default()
    };
    let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(data)) else {
        //没有EXIF信息
        return Ok(info);
    };
    let text = |tag: Tag| -> String {
        exif.get_field(tag, In::PRIMARY)
            .map(|it| it.display_value().to_string().trim_matches('"').trim().to_string())
            .unwrap_or_default()
    };
    let date = match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        Some(_) => text(Tag::DateTimeOriginal),
        None => text(Tag::DateTime),
    };
    let date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&date, "%Y:%m:%d %H:%M:%S"));
    if let Ok(it) = date {
        if let Some(it) = Local.from_local_datetime(&it).earliest() {
            info.date = it.timestamp_millis();
        }
    }
    info.make = text(Tag::Make);
    info.model = text(Tag::Model);
    info.f_number = text(Tag::FNumber);
    info.shutter_speed = text(Tag::ExposureTime);
    info.iso = text(Tag::PhotographicSensitivity).parse().unwrap_or(0);
    info.orientation = exif
        .get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|it| it.value.get_uint(0))
        .unwrap_or(1) as i64;
    if let (Some(lat), Some(long)) = (gps(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef), gps(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef)) {
        info.lat = lat;
        info.long = long;
    }
    Ok(info)
}

/// 读取GPS坐标(度分秒转换成度)
fn gps(exif: &exif::Exif, tag: Tag, ref_tag: Tag) -> Option<f64> {
    let Value::Rational(values) = &exif.get_field(tag, In::PRIMARY)?.value else {
        return None;
    };
    if values.len() < 3 {
        return None;
    }
    let value = values[0].to_f64() + values[1].to_f64() / 60.0 + values[2].to_f64() / 3600.0;
    let negative = exif
        .get_field(ref_tag, In::PRIMARY)
        .map(|it| {
            let text = it.display_value().to_string();
            text.contains('S') || text.contains('W')
        })
        .unwrap_or(false);
    Some(if negative { -value } else { value })
}

/// 解码图片,并按照EXIF的方向信息将图片转正
/// 透明背景填充为白色
pub fn decode(data: &[u8]) -> Result<RgbImage, String> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let orientation = exif::Reader::new()
        .read_from_container(&mut Cursor::new(data))
        .ok()
        .and_then(|it| it.get_field(Tag::Orientation, In::PRIMARY).and_then(|f| f.value.get_uint(0)))
        .unwrap_or(1);
    Ok(to_rgb(apply_orientation(img, orientation)))
}

/// 按照EXIF的方向信息旋转图片
pub fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// 转换成RGB图片,透明部分填充白色
fn to_rgb(img: DynamicImage) -> RgbImage {
    if !img.color().has_alpha() {
        return img.into_rgb8();
    }
    let rgba = img.into_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (src, dst) in rgba.pixels().zip(rgb.pixels_mut()) {
        let alpha = src[3] as u32;
        for i in 0..3 {
            dst[i] = ((src[i] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
        }
    }
    rgb
}

/// 编码成JPEG
pub fn encode_jpeg(img: &RgbImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    JpegEncoder::new_with_quality(&mut buf, quality)
        .encode_image(img)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

/// 转换成JPEG图片
pub fn to_jpeg(data: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    encode_jpeg(&decode(data)?, quality)
}

/// 按比例缩放图片,图片最大边不超过target_max_size
pub fn resize(data: &[u8], target_max_size: u32, quality: u8) -> Result<Vec<u8>, String> {
    let img = decode(data)?;
    let (width, height) = scale_size(img.width(), img.height(), target_max_size);
    if width == img.width() && height == img.height() {
        return encode_jpeg(&img, quality);
    }
    encode_jpeg(&shrink(&img, width, height), quality)
}

/// 按目标宽高比从中间裁剪图片,再缩放到目标尺寸
pub fn crop(data: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, String> {
    let img = decode(data)?;
    let src_scale = img.width() as f64 / img.height() as f64;
    let target_scale = width as f64 / height as f64;
    let (cut_width, cut_height) = if target_scale > src_scale {
        (img.width(), ((img.width() as f64 / target_scale) as u32).max(1))
    } else {
        (((img.height() as f64 * target_scale) as u32).max(1), img.height())
    };
    let x = (img.width() - cut_width) / 2;
    let y = (img.height() - cut_height) / 2;
    let cropped = image::imageops::crop_imm(&img, x, y, cut_width, cut_height).to_image();
    if cut_width > width {
        return encode_jpeg(&shrink(&cropped, width, height), quality);
    }
    encode_jpeg(&cropped, quality)
}

/// 缩小图片,大图先用快速的算法缩小到目标尺寸的2倍,再用高质量的算法缩小
fn shrink(img: &RgbImage, width: u32, height: u32) -> RgbImage {
    if img.width() > width * 4 {
        let temp = image::imageops::thumbnail(img, width * 2, height * 2);
        return image::imageops::resize(&temp, width, height, FilterType::CatmullRom);
    }
    image::imageops::resize(img, width, height, FilterType::CatmullRom)
}

/// 按比例计算缩放后的尺寸
fn scale_size(width: u32, height: u32, target_max_size: u32) -> (u32, u32) {
    if width >= height {
        if width <= target_max_size {
            return (width, height);
        }
        let target_height = (target_max_size as f64 / width as f64 * height as f64) as u32;
        (target_max_size, target_height.max(1))
    } else {
        if height <= target_max_size {
            return (width, height);
        }
        let target_width = (target_max_size as f64 / height as f64 * width as f64) as u32;
        (target_width.max(1), target_max_size)
    }
}
