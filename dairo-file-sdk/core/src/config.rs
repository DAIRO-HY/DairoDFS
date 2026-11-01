/// SDK配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 数据保存目录,缓存索引和缓存文件都保存在这个目录下
    /// 不要使用系统的缓存目录,避免还没有过期就被系统清理
    pub root_dir: String,

    /// 同时缓存的文件数
    pub max_caching_count: usize,

    /// 同时永久保存(下载)的文件数
    pub max_saving_count: usize,

    /// 同时上传的文件数
    pub max_uploading_count: usize,

    /// 网络错误时的重试次数
    pub retry_times: u32,

    /// 连接超时(秒)
    pub connect_timeout: u64,

    /// 读取数据超时(秒),超过这个时间没有收到数据就认为网络中断
    pub read_timeout: u64,

    /// 缓存保存天数,超过这么多天没有访问的缓存会被自动清理,0表示不按时间清理(永久保存的文件不会被清理)
    pub cache_expire_days: u32,

    /// 缓存容量上限(字节),超过时从最久没有访问的缓存开始清理,0表示不限制(永久保存的文件不计算在内)
    pub max_cache_size: u64,
}

impl Config {
    pub fn new(root_dir: impl Into<String>) -> Self {
        Self {
            root_dir: root_dir.into(),
            max_caching_count: 5,
            max_saving_count: 3,
            max_uploading_count: 1,
            retry_times: 5,
            connect_timeout: 10,
            read_timeout: 30,
            cache_expire_days: 90,
            max_cache_size: 0,
        }
    }
}
