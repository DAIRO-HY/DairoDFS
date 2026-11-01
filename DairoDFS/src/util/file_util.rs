use crate::biz_error;
use crate::web::biz_error::BizError;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;
use tokio::io::AsyncReadExt;

/// 文件路径中不允许出现的字符
const ILLEGAL_CHARS: [char; 8] = ['>', '?', '\\', ':', '|', '<', '*', '"'];

/// 文件扩展名 -> ContentType
static CONTENT_TYPE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    include_str!("../../assets/content-type.txt")
        .lines()
        .filter_map(|it| it.trim().split_once(':'))
        .collect()
});

/// 检查文件路径是否合法
pub fn check_path(path: &str) -> Result<(), BizError> {
    if path.contains(ILLEGAL_CHARS) {
        return Err(biz_error!("文件路径不能包含>,?,\\,:,|,<,*,\"字符"));
    }
    if path.contains("//") {
        return Err(biz_error!("文件路径不能包含两个连续的字符/"));
    }
    Ok(())
}

/// 检查文件名是否合法
pub fn check_name(name: &str) -> Result<(), BizError> {
    if name.is_empty() {
        return Err(biz_error!("文件名不能为空"));
    }
    if name.contains('/') {
        return Err(biz_error!("文件名不能包含/"));
    }
    if name == "." || name == ".." {
        return Err(biz_error!("文件名不能是.或.."));
    }
    check_path(name)
}

/// 将路径分割成文件名列表,第一个元素固定为空字符串,代表用户的根目录
/// 例: /a/b => ["", "a", "b"]
pub fn to_sub_names(path: &str) -> Result<Vec<&str>, BizError> {
    check_path(path)?;
    if path.is_empty() {
        return Ok(vec![""]);
    }
    if !path.starts_with('/') {
        return Err(biz_error!("文件路径必须以/开头"));
    }
    Ok(path.strip_suffix('/').unwrap_or(path).split('/').collect())
}

/// 获取路径中的文件名
pub fn file_name(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[index + 1..],
        None => path,
    }
}

/// 获取路径中的上级文件夹
pub fn file_parent(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[..index],
        None => "",
    }
}

/// 获取文件扩展名(小写,不包含点)
pub fn file_ext(name: &str) -> String {
    match name.rfind('.') {
        Some(index) => name[index + 1..].to_lowercase(),
        None => String::new(),
    }
}

/// 通过文件扩展名获取ContentType
pub fn content_type(ext: &str) -> String {
    CONTENT_TYPE_MAP
        .get(ext.to_lowercase().as_str())
        .unwrap_or(&"application/octet-stream")
        .to_string()
}

/// 计算文件的MD5
pub async fn md5(path: &Path) -> std::io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut context = md5::Context::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        context.consume(&buf[..n]);
    }
    Ok(format!("{:x}", context.finalize()))
}
