pub mod alias;
pub mod distributed_push;
pub mod sync_by_log;
pub mod sync_by_table;
pub mod sync_handle;
pub mod sync_http;
pub mod sync_info;
pub mod table_data;
pub mod version;

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

/// 全量同步未完成的标记文件:重置数据之后创建,全量同步成功之后删除
/// 标记存在时本机的文件记录不完整,不能回收存储文件(否则会把还没同步过来的文件当成没有被使用)
fn full_sync_pending_path() -> std::path::PathBuf {
    std::path::PathBuf::from(&application::ARGS.data_dir).join(".full_sync_pending")
}

/// 标记全量同步未完成
pub fn set_full_sync_pending() -> std::io::Result<()> {
    std::fs::write(full_sync_pending_path(), crate::util::time_util::current_millis().to_string())
}

/// 全量同步已经完成
pub fn clear_full_sync_pending() {
    let _ = std::fs::remove_file(full_sync_pending_path());
}

/// 是否有未完成的全量同步
pub fn is_full_sync_pending() -> bool {
    full_sync_pending_path().exists()
}

/// 记录最后一次同步到的日志顺序号的文件
/// 老版本按日志ID记录(sync_last_id.*),转发过来的日志ID不一定比之前的大,级联或双向同步时会漏掉日志
/// 升级之后不再使用老的记录,每台同步主机自动重新全量同步一次
fn last_seq_path(url: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(&application::ARGS.data_dir).join(format!("sync_last_seq.{}", url.md5()))
}

/// 获取最后一次同步到的日志顺序号,没有记录时表示还没有从这台主机全量同步过
pub fn get_last_seq(url: &str) -> Option<i64> {
    std::fs::read_to_string(last_seq_path(url)).ok().and_then(|it| it.trim().parse().ok())
}

/// 保存最后一次同步到的日志顺序号
pub fn save_last_seq(url: &str, seq: i64) -> Result<(), String> {
    std::fs::write(last_seq_path(url), seq.to_string()).map_err(|e| e.to_string())
}

/// 本机的数据版本号的文件:全量同步改变了本机的数据时加1
/// 全量同步过来的数据没有同步日志,级联在本机下面的分机通过日志同步不到,分机发现版本号变了时重新全量同步
fn epoch_path() -> std::path::PathBuf {
    std::path::PathBuf::from(&application::ARGS.data_dir).join(".sync_epoch")
}

/// 本机的数据版本号
pub fn local_epoch() -> i64 {
    std::fs::read_to_string(epoch_path()).ok().and_then(|it| it.trim().parse().ok()).unwrap_or(0)
}

/// 全量同步改变了本机的数据,版本号加1
pub fn bump_local_epoch() -> Result<(), String> {
    std::fs::write(epoch_path(), (local_epoch() + 1).to_string()).map_err(|e| e.to_string())
}

/// 记录最后一次全量同步时主机的数据版本号的文件
fn peer_epoch_path(url: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(&application::ARGS.data_dir).join(format!("sync_epoch.{}", url.md5()))
}

/// 最后一次全量同步时主机的数据版本号
pub fn get_peer_epoch(url: &str) -> Option<i64> {
    std::fs::read_to_string(peer_epoch_path(url)).ok().and_then(|it| it.trim().parse().ok())
}

/// 保存全量同步时主机的数据版本号
pub fn save_peer_epoch(url: &str, epoch: i64) -> Result<(), String> {
    std::fs::write(peer_epoch_path(url), epoch.to_string()).map_err(|e| e.to_string())
}

/// 是否需要从这台主机全量同步:还没有全量同步过,或者主机的数据版本号变了(主机从其他服务器全量同步过来了新的数据)
pub fn need_full_sync(info: &sync_info::SyncInfo) -> bool {
    if get_last_seq(&info.url).is_none() {
        return true;
    }
    match &*info.peer.lock().unwrap() {
        Some(peer) => get_peer_epoch(&info.url) != Some(peer.epoch),
        None => get_peer_epoch(&info.url).is_none(),
    }
}

/// 是否所有的同步主机都已经全量同步完成(没有完成时本机的数据可能不完整)
pub fn all_synced() -> bool {
    sync_info::list().iter().all(|it| !need_full_sync(it))
}

/// 主机的信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {

    // 集群ID
    pub cluster_id: String,

    // 服务器编号
    pub node_no: i64,

    // 是否只读
    pub is_read_only: bool,

    // 主机的当前时间
    pub time: i64,

    // 主机的数据版本号
    #[serde(default)]
    pub epoch: i64,
}

/// 两台服务器时钟相差超过这个时间(毫秒)时提示
const CLOCK_WARN_MILLIS: i64 = 5_000;

/// 两台服务器时钟相差超过这个时间(毫秒)时停止同步:同时修改同一条记录时按修改时间判断先后,时钟相差太多会导致结果不对
const CLOCK_ERROR_MILLIS: i64 = 60_000;

/// 获取主机的信息
pub async fn request_node_info(url: &str) -> Result<NodeInfo, String> {
    let body = sync_http::request(&format!("{}/get_node_info", url)).await?;
    serde_json::from_str(&body).map_err(|e| format!("主机信息格式错误:{}", e))
}

/// 检查主机的信息:服务器编号不能和本机相同,时钟不能相差太多;同时同步集群ID
pub async fn check_node(info: &sync_info::SyncInfo) -> Result<(), String> {
    let start = crate::util::time_util::current_millis();
    let node = request_node_info(&info.url).await?;
    let end = crate::util::time_util::current_millis();
    let config = crate::system_config::get();
    if !config.is_read_only && !node.is_read_only && node.node_no == config.node_no {
        return Err(format!(
            "本机和主机的服务器编号都是{}，双向同步的服务器编号不能相同，请在系统设置里修改其中一台的服务器编号。",
            node.node_no
        ));
    }

    //主机时间加上一半的请求时间,和本机时间比较
    let diff = (node.time + (end - start) / 2 - end).abs();
    if diff > CLOCK_ERROR_MILLIS {
        return Err(format!(
            "本机和主机的时钟相差{}秒，请先校准服务器时间(超过{}秒时停止同步)。",
            diff / 1000,
            CLOCK_ERROR_MILLIS / 1000
        ));
    }
    let warn = if diff > CLOCK_WARN_MILLIS {
        format!("本机和主机的时钟相差{}秒，请校准服务器时间。", diff / 1000)
    } else {
        String::new()
    };
    info.update(|it| it.warn = warn);
    *info.peer.lock().unwrap() = Some(node.clone());

    //只有第一台同步主机需要同步集群ID
    if info.no == 1 && !node.cluster_id.is_empty() && config.cluster_id != node.cluster_id {
        let mut config = (*config).clone();
        config.cluster_id = node.cluster_id;
        crate::system_config::save(config).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 本机要处理哪些文件(生成缩略图等):双向同步时每台服务器只处理自己上传的文件,避免重复生成
/// - return (只处理这个服务器编号上传的文件,小于0时处理所有文件; 是否负责处理其他服务器长时间没有处理的文件和补生成任务)
pub fn handle_scope() -> (i64, bool) {
    let config = crate::system_config::get();
    let infos = sync_info::list();
    if infos.is_empty() {
        return (-1, true);
    }

    //可写的主机里服务器编号最小的负责
    let mut is_leader = true;
    for it in infos {
        match &*it.peer.lock().unwrap() {
            //还不知道主机的信息时,先只处理自己的
            None => is_leader = false,
            Some(peer) if !peer.is_read_only && peer.node_no < config.node_no => is_leader = false,
            Some(_) => {}
        }
    }
    (config.node_no, is_leader)
}
