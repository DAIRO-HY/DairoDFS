/// 缓存方式
/// 同一个文件被多次请求时,取级别最高的方式(永久保存 > 缓存 > 不缓存)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CacheMode {
    /// 不缓存,下载的文件只在本次运行期间有效,SDK下次启动时删除
    None = 0,

    /// 缓存,超过保存期限或者超过缓存容量时会被自动清理
    Cache = 1,

    /// 永久保存(下载功能),不会被自动清理,App重启之后会继续下载
    Save = 2,
}

impl CacheMode {
    pub(crate) fn from_i64(value: i64) -> Self {
        match value {
            2 => CacheMode::Save,
            1 => CacheMode::Cache,
            _ => CacheMode::None,
        }
    }
}

/// 从下载地址生成缓存key的方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    /// 只使用路径部分(去掉域名和参数),默认
    Path,

    /// 使用路径和参数部分(去掉域名)
    PathAndQuery,

    /// 使用完整的地址
    Url,
}

/// 访问文件时的参数(SDK自己的参数,与下载地址里的参数无关)
#[derive(Debug, Clone)]
pub struct Options {
    /// 缓存方式
    pub cache: CacheMode,

    /// 缓存分组,用于分组统计和清理,例如:music、image
    pub group: String,

    /// 命名空间,用来区分文件来自哪个服务器,DFS传集群ID,不传时使用协议和域名
    pub namespace: Option<String>,

    /// 生成缓存key的方式
    pub key_mode: KeyMode,

    /// 自定义缓存key,指定之后忽略key_mode
    pub cache_key: Option<String>,

    /// 下载时附加的请求头,例如:Authorization
    pub headers: Vec<(String, String)>,

    /// 优先级,数值越大越先下载
    pub priority: i32,

    /// 文件名,用于确定缓存文件的扩展名和显示
    pub name: Option<String>,

    /// 文件的MD5,指定之后下载完成时会校验
    pub md5: Option<String>,

    /// 缩略图地址,用于下载列表显示
    pub thumb: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cache: CacheMode::Cache,
            group: "default".to_string(),
            namespace: None,
            key_mode: KeyMode::Path,
            cache_key: None,
            headers: Vec::new(),
            priority: 0,
            name: None,
            md5: None,
            thumb: None,
        }
    }
}
