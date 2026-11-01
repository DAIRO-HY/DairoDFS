use crate::application;
use arc_swap::ArcSwap;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

/// 系统配置(保存在data目录下的system.json,字段名沿用老版本的格式,老版本的配置文件可以直接读取)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SystemConfig {
    /// 记录同步日志
    pub open_sql_log: bool,

    /// 将当前服务器设置为只读,仅作为备份使用
    pub is_read_only: bool,

    /// 文件保存文件夹列表
    pub save_folder_list: Vec<String>,

    /// 同步域名
    pub sync_domains: Vec<String>,

    /// 分机与主机同步连接票据
    pub distributed_token: String,

    /// 回收站超时(单位：天)
    pub trash_timeout: i64,

    /// 删除没有被使用的文件超时设置(单位：天)
    pub delete_storage_timeout: i64,

    /// 缩略图最大边尺寸
    pub thumb_max_size: i64,

    /// 忽略本机同步错误
    pub ignore_sync_error: bool,

    /// 数据库备份天数
    pub db_backup_expire_day: i64,

    /// 集群ID,同一个分布式集群里的服务器相同(分机从第一台同步主机获取),客户端用来判断文件是否来自同一个集群
    pub cluster_id: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            open_sql_log: false,
            is_read_only: false,
            save_folder_list: vec![application::ARGS.data_dir.clone()],
            sync_domains: Vec::new(),
            distributed_token: Alphanumeric.sample_string(&mut rand::rng(), 32),
            trash_timeout: 30,
            delete_storage_timeout: 30,
            thumb_max_size: 360,
            ignore_sync_error: false,
            db_backup_expire_day: 30,
            cluster_id: String::new(),
        }
    }
}

/// 当前系统配置
static INSTANCE: LazyLock<ArcSwap<SystemConfig>> = LazyLock::new(|| ArcSwap::new(Arc::new(load())));

/// 加载系统配置,配置文件有问题时直接退出,避免用默认配置覆盖掉用户的配置
pub fn init() {
    LazyLock::force(&INSTANCE);
}

/// 获取当前系统配置
pub fn get() -> Arc<SystemConfig> {
    INSTANCE.load_full()
}

/// 保存系统配置
pub fn save(config: SystemConfig) -> std::io::Result<()> {
    write(&config)?;
    INSTANCE.store(Arc::new(config));
    Ok(())
}

/// 从配置文件读取,文件不存在时创建一份默认配置
fn load() -> SystemConfig {
    let path = application::SYSTEM_JSON_PATH.as_path();
    let mut config = if path.exists() {
        let data = std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("-->读取配置文件:{}失败: {}", path.display(), e);
            std::process::exit(1);
        });
        serde_json::from_slice(&data).unwrap_or_else(|e| {
            eprintln!("-->配置文件:{}格式错误: {}", path.display(), e);
            std::process::exit(1);
        })
    } else {
        SystemConfig::default()
    };

    //新建的配置文件或者老版本的配置文件没有集群ID
    if !path.exists() || config.cluster_id.is_empty() {
        if config.cluster_id.is_empty() {
            config.cluster_id = Alphanumeric.sample_string(&mut rand::rng(), 32);
        }
        if let Err(e) = write(&config) {
            eprintln!("-->保存配置文件:{}失败: {}", path.display(), e);
            std::process::exit(1);
        }
    }
    config
}

/// 写入配置文件(先写临时文件再改名,避免写到一半断电导致配置文件损坏)
fn write(config: &SystemConfig) -> std::io::Result<()> {
    let path = application::SYSTEM_JSON_PATH.as_path();
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, serde_json::to_vec(config).unwrap())?;
    std::fs::rename(&temp_path, path)
}
