//! 导出给Dart的数据类型,和核心库的类型一一对应

use dairo_file_sdk as core;
use flutter_rust_bridge::frb;
use std::collections::HashMap;

/// SDK配置
#[frb(non_opaque)]
#[derive(Debug, Clone)]
pub struct SdkConfig {
    /// 数据保存目录,缓存索引和缓存文件都保存在这个目录下(建议使用path_provider的getApplicationSupportDirectory)
    pub root_dir: String,

    /// 同时缓存的文件数
    #[frb(default = 5)]
    pub max_caching_count: u32,

    /// 同时永久保存(下载)的文件数
    #[frb(default = 3)]
    pub max_saving_count: u32,

    /// 同时上传的文件数
    #[frb(default = 1)]
    pub max_uploading_count: u32,

    /// 网络错误时的重试次数
    #[frb(default = 5)]
    pub retry_times: u32,

    /// 连接超时(秒)
    #[frb(default = 10)]
    pub connect_timeout: i64,

    /// 读取数据超时(秒)
    #[frb(default = 30)]
    pub read_timeout: i64,

    /// 缓存保存天数,0表示不按时间清理
    #[frb(default = 90)]
    pub cache_expire_days: u32,

    /// 缓存容量上限(字节),0表示不限制
    #[frb(default = 0)]
    pub max_cache_size: i64,
}

impl From<SdkConfig> for core::Config {
    fn from(it: SdkConfig) -> Self {
        let mut config = core::Config::new(it.root_dir);
        config.max_caching_count = it.max_caching_count as usize;
        config.max_saving_count = it.max_saving_count as usize;
        config.max_uploading_count = it.max_uploading_count as usize;
        config.retry_times = it.retry_times;
        config.connect_timeout = it.connect_timeout as u64;
        config.read_timeout = it.read_timeout as u64;
        config.cache_expire_days = it.cache_expire_days;
        config.max_cache_size = it.max_cache_size.max(0) as u64;
        config
    }
}

/// 缓存方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// 不缓存,SDK下次启动时删除
    None,

    /// 缓存,超过保存期限或者超过缓存容量时会被自动清理
    Cache,

    /// 永久保存(下载功能),不会被自动清理
    Save,
}

impl From<CacheMode> for core::CacheMode {
    fn from(it: CacheMode) -> Self {
        match it {
            CacheMode::None => core::CacheMode::None,
            CacheMode::Cache => core::CacheMode::Cache,
            CacheMode::Save => core::CacheMode::Save,
        }
    }
}

impl From<core::CacheMode> for CacheMode {
    fn from(it: core::CacheMode) -> Self {
        match it {
            core::CacheMode::None => CacheMode::None,
            core::CacheMode::Cache => CacheMode::Cache,
            core::CacheMode::Save => CacheMode::Save,
        }
    }
}

/// 从下载地址生成缓存key的方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    /// 只使用路径部分(去掉域名和参数)
    Path,

    /// 使用路径和参数部分(去掉域名)
    PathAndQuery,

    /// 使用完整的地址
    Url,
}

/// 访问文件时的参数(SDK自己的参数,与下载地址里的参数无关)
#[frb(non_opaque)]
#[derive(Debug, Clone)]
pub struct Options {
    /// 缓存方式
    #[frb(default = "CacheMode.cache")]
    pub cache: CacheMode,

    /// 缓存分组,用于分组统计和清理,例如:music、image
    #[frb(default = "default")]
    pub group: String,

    /// 命名空间,用来区分文件来自哪个服务器,DFS传集群ID,不传时使用协议和域名
    pub namespace: Option<String>,

    /// 生成缓存key的方式
    #[frb(default = "KeyMode.path")]
    pub key_mode: KeyMode,

    /// 自定义缓存key,指定之后忽略keyMode
    pub cache_key: Option<String>,

    /// 下载时附加的请求头,例如:Authorization
    #[frb(default = "const {}")]
    pub headers: HashMap<String, String>,

    /// 优先级,数值越大越先下载
    #[frb(default = 0)]
    pub priority: i32,

    /// 文件名,用于确定缓存文件的扩展名和显示
    pub name: Option<String>,

    /// 文件的MD5,指定之后下载完成时会校验
    pub md5: Option<String>,

    /// 缩略图地址,用于下载列表显示
    pub thumb: Option<String>,
}

impl From<Options> for core::Options {
    fn from(it: Options) -> Self {
        core::Options {
            cache: it.cache.into(),
            group: it.group,
            namespace: it.namespace,
            key_mode: match it.key_mode {
                KeyMode::Path => core::KeyMode::Path,
                KeyMode::PathAndQuery => core::KeyMode::PathAndQuery,
                KeyMode::Url => core::KeyMode::Url,
            },
            cache_key: it.cache_key,
            headers: it.headers.into_iter().collect(),
            priority: it.priority,
            name: it.name,
            md5: it.md5,
            thumb: it.thumb,
        }
    }
}

/// SDK错误
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SdkError {
    /// 地址过期或没有权限(状态码401/403),下载时需要换一个新的地址重新请求,上传时需要更新登录票据
    #[error("{msg}")]
    UrlExpired { status: u16, msg: String },

    /// 文件不存在
    #[error("{msg}")]
    NotFound { msg: String },

    /// 服务器返回了其他错误状态码
    #[error("{msg}")]
    Http { status: u16, msg: String },

    /// 服务器返回的错误信息
    #[error("{msg}")]
    Server { msg: String },

    /// 上传的数据来源读取失败
    #[error("{msg}")]
    Source { msg: String },

    /// 网络错误(重试次数用完之后仍然失败)
    #[error("{msg}")]
    Network { msg: String },

    /// 文件读写错误
    #[error("{msg}")]
    Io { msg: String },

    /// 下载完成的文件MD5与期望的不一致
    #[error("{msg}")]
    Md5Mismatch { msg: String },

    /// 地址格式不正确
    #[error("{msg}")]
    InvalidUrl { msg: String },

    /// 缓存索引数据库错误
    #[error("{msg}")]
    Db { msg: String },

    /// 还没有设置DFS账号
    #[error("{msg}")]
    NoAccount { msg: String },

    /// 已经被取消
    #[error("{msg}")]
    Cancelled { msg: String },
}

impl From<core::SdkError> for SdkError {
    fn from(e: core::SdkError) -> Self {
        let msg = e.to_string();
        match e {
            core::SdkError::UrlExpired(status) => SdkError::UrlExpired { status, msg },
            core::SdkError::NotFound => SdkError::NotFound { msg },
            core::SdkError::Http(status) => SdkError::Http { status, msg },
            core::SdkError::Server(_) => SdkError::Server { msg },
            core::SdkError::Source(_) => SdkError::Source { msg },
            core::SdkError::Network(_) => SdkError::Network { msg },
            core::SdkError::Io(_) => SdkError::Io { msg },
            core::SdkError::Md5Mismatch => SdkError::Md5Mismatch { msg },
            core::SdkError::InvalidUrl(_) => SdkError::InvalidUrl { msg },
            core::SdkError::Db(_) => SdkError::Db { msg },
            core::SdkError::NoAccount => SdkError::NoAccount { msg },
            core::SdkError::Cancelled => SdkError::Cancelled { msg },
        }
    }
}

/// 下载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    Waiting,
    Downloading,
    Paused,
    Error,
    Finish,
}

impl From<core::DownloadState> for DownloadState {
    fn from(it: core::DownloadState) -> Self {
        match it {
            core::DownloadState::Waiting => DownloadState::Waiting,
            core::DownloadState::Downloading => DownloadState::Downloading,
            core::DownloadState::Paused => DownloadState::Paused,
            core::DownloadState::Error => DownloadState::Error,
            core::DownloadState::Finish => DownloadState::Finish,
        }
    }
}

impl From<DownloadState> for core::DownloadState {
    fn from(it: DownloadState) -> Self {
        match it {
            DownloadState::Waiting => core::DownloadState::Waiting,
            DownloadState::Downloading => core::DownloadState::Downloading,
            DownloadState::Paused => core::DownloadState::Paused,
            DownloadState::Error => core::DownloadState::Error,
            DownloadState::Finish => core::DownloadState::Finish,
        }
    }
}

/// 上传状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadState {
    Waiting,
    Uploading,
    Paused,
    Error,
    Finish,

    /// 自己的文件里已经有相同的文件,跳过
    Skipped,
}

impl From<core::UploadState> for UploadState {
    fn from(it: core::UploadState) -> Self {
        match it {
            core::UploadState::Waiting => UploadState::Waiting,
            core::UploadState::Uploading => UploadState::Uploading,
            core::UploadState::Paused => UploadState::Paused,
            core::UploadState::Error => UploadState::Error,
            core::UploadState::Finish => UploadState::Finish,
            core::UploadState::Skipped => UploadState::Skipped,
        }
    }
}

impl From<UploadState> for core::UploadState {
    fn from(it: UploadState) -> Self {
        match it {
            UploadState::Waiting => core::UploadState::Waiting,
            UploadState::Uploading => core::UploadState::Uploading,
            UploadState::Paused => core::UploadState::Paused,
            UploadState::Error => core::UploadState::Error,
            UploadState::Finish => core::UploadState::Finish,
            UploadState::Skipped => core::UploadState::Skipped,
        }
    }
}

/// SDK事件
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// 下载状态改变
    StateChanged { id: String, mode: CacheMode, state: DownloadState, error: Option<String> },

    /// 下载进度
    Progress { id: String, mode: CacheMode, size: Option<i64>, downloaded: i64, speed: i64 },

    /// 缓存被删除(包括自动清理)
    Removed { ids: Vec<String> },

    /// 上传状态改变
    UploadStateChanged { id: String, state: UploadState, error: Option<String> },

    /// 上传进度,hashing为true时正在计算MD5
    UploadProgress { id: String, hashing: bool, size: i64, done: i64, speed: i64 },

    /// 上传记录被删除
    UploadRemoved { ids: Vec<String> },
}

impl From<core::Event> for Event {
    fn from(it: core::Event) -> Self {
        match it {
            core::Event::StateChanged { id, mode, state, error } => {
                Event::StateChanged { id, mode: mode.into(), state: state.into(), error }
            }
            core::Event::Progress { id, mode, size, downloaded, speed } => Event::Progress {
                id,
                mode: mode.into(),
                size: size.map(|it| it as i64),
                downloaded: downloaded as i64,
                speed: speed as i64,
            },
            core::Event::Removed { ids } => Event::Removed { ids },
            core::Event::UploadStateChanged { id, state, error } => {
                Event::UploadStateChanged { id: id.to_string(), state: state.into(), error }
            }
            core::Event::UploadProgress { id, hashing, size, done, speed } => Event::UploadProgress {
                id: id.to_string(),
                hashing,
                size: size as i64,
                done: done as i64,
                speed: speed as i64,
            },
            core::Event::UploadRemoved { ids } => Event::UploadRemoved { ids: ids.iter().map(|it| it.to_string()).collect() },
        }
    }
}

/// 下载进度和结果
#[derive(Debug, Clone, PartialEq)]
pub enum FetchEvent {
    /// 下载进度,size为文件大小,未知时为null
    Progress { size: Option<i64>, downloaded: i64 },

    /// 下载完成,path为缓存文件路径
    Success { path: String },

    /// 下载失败
    Error { error: SdkError },
}

/// 缓存信息
#[derive(Debug, Clone, PartialEq)]
pub struct CacheInfo {
    pub id: String,
    pub url: String,
    pub namespace: String,
    pub key: String,
    pub group: String,
    pub mode: CacheMode,
    pub name: Option<String>,
    pub thumb: Option<String>,

    /// 文件大小,未知时为null
    pub size: Option<i64>,
    pub downloaded: i64,
    pub state: DownloadState,
    pub error: Option<String>,

    /// 创建时间(毫秒)
    pub create_date: i64,

    /// 下载完成时间(毫秒)
    pub finish_date: Option<i64>,

    /// 最后访问时间(毫秒)
    pub access_date: i64,

    /// 缓存文件路径,下载完成时才有
    pub path: Option<String>,
}

impl From<core::CacheInfo> for CacheInfo {
    fn from(it: core::CacheInfo) -> Self {
        Self {
            id: it.id,
            url: it.url,
            namespace: it.namespace,
            key: it.key,
            group: it.group,
            mode: it.mode.into(),
            name: it.name,
            thumb: it.thumb,
            size: it.size.map(|it| it as i64),
            downloaded: it.downloaded as i64,
            state: it.state.into(),
            error: it.error,
            create_date: it.create_date,
            finish_date: it.finish_date,
            access_date: it.access_date,
            path: it.path,
        }
    }
}

/// 缓存查询条件
#[frb(non_opaque)]
#[derive(Debug, Clone)]
pub struct CacheQuery {
    pub group: Option<String>,
    pub mode: Option<CacheMode>,
    pub state: Option<DownloadState>,
    #[frb(default = 0)]
    pub offset: i64,
    #[frb(default = 100)]
    pub limit: i64,
}

impl From<CacheQuery> for core::CacheQuery {
    fn from(it: CacheQuery) -> Self {
        Self {
            group: it.group,
            mode: it.mode.map(Into::into),
            state: it.state.map(Into::into),
            offset: it.offset.max(0) as u64,
            limit: it.limit.max(0) as u64,
        }
    }
}

/// 分组统计
#[derive(Debug, Clone, PartialEq)]
pub struct GroupStats {
    pub group: String,
    pub count: i64,
    pub size: i64,
}

/// 要上传的文件
#[frb(non_opaque)]
#[derive(Debug, Clone)]
pub struct UploadItem {
    /// 本地文件路径
    pub path: String,

    /// 上传到DFS的文件路径(包含文件名),例如:/相册/1.jpg
    pub dfs_path: String,

    /// 文件MD5,已经计算过时传入
    pub md5: Option<String>,

    /// 文件类型,不指定时DFS按扩展名判断
    pub content_type: Option<String>,

    /// 缩略图地址,用于上传列表显示
    pub thumb: Option<String>,

    /// 自己的文件里已经有相同的文件时跳过
    #[frb(default = false)]
    pub skip_if_exists: bool,
}

impl From<UploadItem> for core::UploadItem {
    fn from(it: UploadItem) -> Self {
        core::UploadItem {
            source: core::UploadSource::File(it.path),
            dfs_path: it.dfs_path,
            md5: it.md5,
            content_type: it.content_type,
            thumb: it.thumb,
            skip_if_exists: it.skip_if_exists,
        }
    }
}

/// 上传信息
#[derive(Debug, Clone, PartialEq)]
pub struct UploadInfo {
    /// 上传ID
    pub id: String,
    pub account: String,

    /// 本地文件路径(平台提供数据时为数据来源ID)
    pub source: String,
    pub dfs_path: String,
    pub name: String,
    pub size: Option<i64>,
    pub uploaded: i64,
    pub md5: Option<String>,
    pub thumb: Option<String>,
    pub state: UploadState,
    pub error: Option<String>,
    pub create_date: i64,
    pub finish_date: Option<i64>,
}

impl From<core::UploadInfo> for UploadInfo {
    fn from(it: core::UploadInfo) -> Self {
        Self {
            id: it.id.to_string(),
            account: it.account,
            source: match it.source {
                core::UploadSource::File(it) | core::UploadSource::Custom(it) => it,
            },
            dfs_path: it.dfs_path,
            name: it.name,
            size: it.size.map(|it| it as i64),
            uploaded: it.uploaded as i64,
            md5: it.md5,
            thumb: it.thumb,
            state: it.state.into(),
            error: it.error,
            create_date: it.create_date,
            finish_date: it.finish_date,
        }
    }
}

/// 上传列表查询条件
#[frb(non_opaque)]
#[derive(Debug, Clone)]
pub struct UploadQuery {
    /// 账号,null时查询所有账号
    pub account: Option<String>,
    pub state: Option<UploadState>,
    #[frb(default = 0)]
    pub offset: i64,
    #[frb(default = 100)]
    pub limit: i64,
}

impl From<UploadQuery> for core::UploadQuery {
    fn from(it: UploadQuery) -> Self {
        Self { account: it.account, state: it.state.map(Into::into), offset: it.offset.max(0) as u64, limit: it.limit.max(0) as u64 }
    }
}
