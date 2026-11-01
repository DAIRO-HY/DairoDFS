use std::fmt;

/// SDK错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdkError {
    /// 地址过期或没有权限(状态码401/403),下载时平台需要换一个新的地址重新请求,上传时需要更新登录票据
    UrlExpired(u16),

    /// 文件不存在(状态码404)
    NotFound,

    /// 服务器返回了其他错误状态码
    Http(u16),

    /// 服务器返回的错误信息
    Server(String),

    /// 上传的数据来源读取失败
    Source(String),

    /// 网络错误(重试次数用完之后仍然失败)
    Network(String),

    /// 文件读写错误
    Io(String),

    /// 下载完成的文件MD5与期望的不一致
    Md5Mismatch,

    /// 下载地址格式不正确
    InvalidUrl(String),

    /// 缓存索引数据库错误
    Db(String),

    /// 还没有设置DFS账号
    NoAccount,

    /// 已经被取消
    Cancelled,
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::UrlExpired(status) => write!(f, "地址已过期或者没有权限(登录票据可能已经失效):{}", status),
            SdkError::NotFound => write!(f, "文件不存在"),
            SdkError::Http(status) => write!(f, "服务器错误:{}", status),
            SdkError::Server(msg) => write!(f, "{}", msg),
            SdkError::Source(msg) => write!(f, "读取上传数据失败:{}", msg),
            SdkError::Network(msg) => write!(f, "网络错误:{}", msg),
            SdkError::Io(msg) => write!(f, "文件读写错误:{}", msg),
            SdkError::Md5Mismatch => write!(f, "文件MD5校验失败"),
            SdkError::InvalidUrl(url) => write!(f, "下载地址格式不正确:{}", url),
            SdkError::Db(msg) => write!(f, "缓存索引错误:{}", msg),
            SdkError::NoAccount => write!(f, "还没有设置DFS账号"),
            SdkError::Cancelled => write!(f, "已取消"),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
    fn from(e: std::io::Error) -> Self {
        SdkError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for SdkError {
    fn from(e: rusqlite::Error) -> Self {
        SdkError::Db(e.to_string())
    }
}

pub type SdkResult<T> = Result<T, SdkError>;
