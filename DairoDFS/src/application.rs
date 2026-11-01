use clap::Parser;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use tokio::sync::Notify;

/// 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 基准时间戳,生成用户文件访问路径前缀时用当前时间戳减去此时间戳(与老版本一致,不能修改)
pub const BASE_TIME: i64 = 1699177026571;

/// 数字转换成短文本支持的字符(与老版本一致,不能修改)
pub const SHORT_CHAR: &str = "0Mkhc7EingwxJYtPdUmWGHeV3ND5KRACb4rBXlO6f91syvIuqoZQLa2FTS8zpj";

/// 同一个用户允许登录的客户端数量
pub const USER_TOKEN_LIMIT: usize = 10;

/// 用来接收关闭通知的全局异步通知器
pub static SHUTDOWN_NOTIFY: Notify = Notify::const_new();

/// 标记Web服务监听是否已经退出
pub static IS_WEB_DROP: AtomicBool = AtomicBool::new(false);

/// 程序启动参数
pub static ARGS: LazyLock<Argument> = LazyLock::new(|| Argument::parse());

/// 开发时使用工作空间根目录下的data目录(由build.rs注入)
#[cfg(debug_assertions)]
const DEFAULT_DATA_DIR: &str = env!("DAIRO_DFS_DEV_DATA_DIR");

/// 发布时使用程序启动目录下的data目录
#[cfg(not(debug_assertions))]
const DEFAULT_DATA_DIR: &str = "./data";

/// 系统配置文件路径
pub static SYSTEM_JSON_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("system.json"));

/// 临时文件目录
pub static TEMP_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("temp"));

#[derive(Parser, Debug)]
#[command(name = "DairoDFS", version, about = "DairoDFS")]
pub struct Argument {
    /// WEB端口(老版本的参数名是--port,继续支持)
    #[arg(long, alias = "port", env = "DAIRO_DFS_WEB_PORT", default_value = "8031")]
    pub web_port: u16,

    /// 数据保存目录(数据库文件、配置文件、默认的文件存储目录都在这个目录下)
    #[arg(long, env = "DAIRO_DFS_DATA_DIR", default_value = DEFAULT_DATA_DIR)]
    pub data_dir: String,

    /// 老版本的日志参数,已经不再使用,保留是为了老版本的启动脚本还能正常启动
    #[arg(long, hide = true)]
    pub log_out_type: Option<String>,

    /// 老版本的日志参数,已经不再使用
    #[arg(long, hide = true)]
    pub log_level: Option<String>,

    /// 老版本的开发模式参数,已经不再使用
    #[arg(long, hide = true)]
    pub is_dev: Option<String>,
}

/// ffmpeg安装目录(与老版本一致)
pub static FFMPEG_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("ffmpeg"));

/// ffprobe安装目录(与老版本一致)
pub static FFPROBE_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("ffprobe"));

/// libraw安装目录(与老版本一致)
pub static LIBRAW_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("libraw"));

/// imagemagick安装目录(与老版本一致)
pub static IMAGEMAGICK_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("imagemagick"));

/// exiftool安装目录(与老版本一致)
pub static EXIFTOOL_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("exiftool"));
