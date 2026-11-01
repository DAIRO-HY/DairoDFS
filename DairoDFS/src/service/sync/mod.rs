pub mod distributed_push;
pub mod sync_by_log;
pub mod sync_by_table;
pub mod sync_handle;
pub mod sync_http;
pub mod sync_info;
pub mod table_data;

use crate::application;
use crate::extension::string::StringExtension;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tokio::task::JoinHandle;

/// 数据同步锁,日志同步和全量同步不能同时执行
pub static SYNC_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// 标记是否正在全量同步中
pub static IS_TABLE_SYNCING: AtomicBool = AtomicBool::new(false);

/// 标记是否正在日志同步中
pub static IS_LOG_SYNCING: AtomicBool = AtomicBool::new(false);

/// 正在监听的日志同步任务
static LISTEN_TASKS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

/// 按照系统配置重新加载同步主机,并开始监听日志
pub fn listen_all() {
    stop_all();
    start_listen(sync_info::reload());
}

/// 开始监听日志
pub fn start_listen(infos: Vec<std::sync::Arc<sync_info::SyncInfo>>) {
    let mut tasks = LISTEN_TASKS.lock().unwrap();
    for info in infos {
        tasks.push(tokio::spawn(sync_by_log::listen(info)));
    }
}

/// 停止所有的日志监听
pub fn stop_all() {
    for it in sync_info::list() {
        it.is_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    for it in LISTEN_TASKS.lock().unwrap().drain(..) {
        it.abort();
    }
}

/// 记录最后一次同步到的日志ID的文件(与老版本一致)
fn last_id_path(url: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(&application::ARGS.data_dir).join(format!("sync_last_id.{}", url.md5()))
}

/// 获取最后一次同步到的日志ID
pub fn get_last_id(url: &str) -> i64 {
    std::fs::read_to_string(last_id_path(url))
        .ok()
        .and_then(|it| it.trim().parse().ok())
        .unwrap_or(0)
}

/// 保存最后一次同步到的日志ID
pub fn save_last_id(url: &str, last_id: i64) -> Result<(), String> {
    std::fs::write(last_id_path(url), last_id.to_string()).map_err(|e| e.to_string())
}

/// 将本机的集群ID设置成和同步主机一致
pub async fn sync_cluster_id(info: &sync_info::SyncInfo) -> Result<(), String> {
    let cluster_id = sync_http::request(&format!("{}/get_cluster_id", info.url)).await?;
    let cluster_id = cluster_id.trim();
    if cluster_id.is_empty() {
        return Err("主机返回的集群ID为空".to_string());
    }
    let config = crate::system_config::get();
    if config.cluster_id == cluster_id {
        return Ok(());
    }
    let mut config = (*config).clone();
    config.cluster_id = cluster_id.to_string();
    crate::system_config::save(config).map_err(|e| e.to_string())
}
