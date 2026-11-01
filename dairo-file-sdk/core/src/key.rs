use crate::error::{SdkError, SdkResult};
use crate::options::{KeyMode, Options};
use reqwest::Url;

/// 缓存key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    /// 命名空间
    pub namespace: String,

    /// 命名空间内的key
    pub key: String,

    /// 缓存ID,md5(命名空间 + key),也用作缓存文件名
    pub id: String,
}

/// 根据下载地址和参数生成缓存key
pub fn resolve(url: &str, options: &Options) -> SdkResult<(Url, CacheKey)> {
    let parsed = Url::parse(url).map_err(|_| SdkError::InvalidUrl(url.to_string()))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(SdkError::InvalidUrl(url.to_string()));
    }
    let namespace = match &options.namespace {
        Some(it) if !it.is_empty() => it.clone(),
        _ => parsed.origin().ascii_serialization(),
    };
    let key = match &options.cache_key {
        Some(it) if !it.is_empty() => it.clone(),
        _ => match options.key_mode {
            KeyMode::Path => parsed.path().to_string(),
            KeyMode::PathAndQuery => match parsed.query() {
                Some(query) => format!("{}?{}", parsed.path(), query),
                None => parsed.path().to_string(),
            },
            KeyMode::Url => parsed.as_str().to_string(),
        },
    };
    let id = format!("{:x}", md5::compute(format!("{}\n{}", namespace, key)));
    Ok((parsed, CacheKey { namespace, key, id }))
}

/// 获取缓存文件的扩展名,优先使用文件名的扩展名,其次使用下载地址最后一段的扩展名
pub fn ext(url: &Url, name: Option<&str>) -> String {
    let from_name = name.and_then(ext_of);
    let from_url = || url.path_segments().and_then(|mut it| it.next_back()).and_then(ext_of);
    from_name.or_else(from_url).unwrap_or_default()
}

/// 取扩展名,只接受较短的字母数字,避免把奇怪的路径当成扩展名
fn ext_of(name: &str) -> Option<String> {
    let (_, ext) = name.rsplit_once('.')?;
    if ext.is_empty() || ext.len() > 10 || !ext.chars().all(|it| it.is_ascii_alphanumeric()) {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mode() {
        let url = "https://a.com:8031/api/files/download/1/a.mp4?_token=x&wait=1";
        let (_, path) = resolve(url, &Options::default()).unwrap();
        assert_eq!(path.namespace, "https://a.com:8031");
        assert_eq!(path.key, "/api/files/download/1/a.mp4");

        let (_, query) = resolve(url, &Options { key_mode: KeyMode::PathAndQuery, ..Default::default() }).unwrap();
        assert_eq!(query.key, "/api/files/download/1/a.mp4?_token=x&wait=1");

        //同一个集群的不同服务器,命名空间相同时key相同
        let ns = Options { namespace: Some("cluster".to_string()), ..Default::default() };
        let (_, a) = resolve("http://192.168.1.2:8031/d/x/1.jpg?t=1", &ns).unwrap();
        let (_, b) = resolve("https://dfs.com/d/x/1.jpg?t=2", &ns).unwrap();
        assert_eq!(a.id, b.id);

        //不同集群凑巧路径一样
        let (_, c) = resolve("https://dfs.com/d/x/1.jpg", &Options { namespace: Some("other".to_string()), ..Default::default() }).unwrap();
        assert_ne!(a.id, c.id);

        let (_, custom) = resolve(url, &Options { cache_key: Some("k".to_string()), ..Default::default() }).unwrap();
        assert_eq!(custom.key, "k");
        assert!(resolve("ftp://a.com/1", &Options::default()).is_err());
    }

    #[test]
    fn file_ext() {
        let url = Url::parse("http://a.com/d/1/Song.MP3?x=1").unwrap();
        assert_eq!(ext(&url, None), "mp3");
        assert_eq!(ext(&url, Some("a.flac")), "flac");
        assert_eq!(ext(&Url::parse("http://a.com/api/download/12").unwrap(), None), "");
    }
}
