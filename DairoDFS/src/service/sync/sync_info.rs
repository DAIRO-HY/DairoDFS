use crate::extension::number::NumberExtension;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

/// 同步状态:待机中
pub const STATE_IDLE: i64 = 0;

/// 同步状态:同步中
pub const STATE_SYNCING: i64 = 1;

/// 同步状态:同步错误
pub const STATE_ERROR: i64 = 2;

/// 当前所有同步主机的信息
static INFOS: LazyLock<RwLock<Vec<Arc<SyncInfo>>>> = LazyLock::new(|| RwLock::new(Vec::new()));

/// 一台同步主机的信息
#[derive(Debug)]
pub struct SyncInfo {

    // 编号
    pub no: usize,

    // 主机端同步连接(主机的同步地址 + "/" + 本机的同步票据),同时也是同步日志的来源标记
    pub url: String,

    // 标记是否已经停止
    pub is_stop: AtomicBool,

    // 同步状态
    pub status: Mutex<SyncStatus>,
}

/// 同步状态
#[derive(Debug, Default, Clone)]
pub struct SyncStatus {

    // 同步状态 0：待机中 1：同步中 2：同步错误
    pub state: i64,

    // 同步消息
    pub msg: String,

    // 要同步的数据总量
    pub count: i64,

    // 已经同步的数据量
    pub sync_count: i64,

    // 最后一次同步完成时间
    pub last_time: i64,

    // 最后一次心跳时间
    pub last_heart_time: i64,
}

impl SyncInfo {
    /// 修改同步状态
    pub fn update(&self, f: impl FnOnce(&mut SyncStatus)) {
        f(&mut self.status.lock().unwrap());
    }

    /// 设置同步消息
    pub fn msg(&self, msg: impl Into<String>) {
        self.status.lock().unwrap().msg = msg.into();
    }

    /// 标记同步出错
    pub fn error(&self, msg: impl Into<String>) {
        self.update(|it| {
            it.state = STATE_ERROR;
            it.msg = msg.into();
        });
    }

    /// 是否已经停止
    pub fn is_stop(&self) -> bool {
        self.is_stop.load(Ordering::Relaxed)
    }

    /// 获取当前状态
    pub fn status(&self) -> SyncStatus {
        self.status.lock().unwrap().clone()
    }
}

/// 获取所有同步主机
pub fn list() -> Vec<Arc<SyncInfo>> {
    INFOS.read().unwrap().clone()
}

/// 通过同步日志的来源找到同步主机
pub fn find(url: &str) -> Option<Arc<SyncInfo>> {
    INFOS.read().unwrap().iter().find(|it| it.url == url).cloned()
}

/// 按照系统配置重新加载同步主机,之前的同步主机全部标记为停止
pub fn reload() -> Vec<Arc<SyncInfo>> {
    let config = crate::system_config::get();
    let mut infos = INFOS.write().unwrap();
    for it in infos.iter() {
        it.is_stop.store(true, Ordering::Relaxed);
    }
    *infos = config
        .sync_domains
        .iter()
        .enumerate()
        .map(|(i, domain)| {
            Arc::new(SyncInfo {
                no: i + 1,
                url: format!("{}/{}", domain.trim_end_matches('/'), config.distributed_token),
                is_stop: AtomicBool::new(false),
                status: Mutex::new(SyncStatus {
                    msg: "等待同步中".to_string(),
                    ..Default::default()
                }),
            })
        })
        .collect();
    infos.clone()
}

/// 是否有同步失败的主机
pub fn has_error() -> bool {
    list().iter().any(|it| it.status().state == STATE_ERROR)
}

/// 同步主机信息(返回给前端)
#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncInfoView {

    // 编号
    pub no: usize,

    // 主机地址(不显示票据)
    pub url: String,

    // 同步状态 0：待机中 1：同步中 2：同步错误
    pub state: i64,

    // 同步消息
    pub msg: String,

    // 同步进度
    pub progress: String,

    // 最后一次同步完成时间
    pub last_time: String,

    // 最后一次心跳时间
    pub last_heart_time: String,
}

/// 获取所有同步主机的信息(返回给前端)
pub fn views() -> Vec<SyncInfoView> {
    list()
        .iter()
        .map(|it| {
            let status = it.status();

            //只显示协议和主机部分,不显示票据
            let host = it.url.split('/').take(3).collect::<Vec<_>>().join("/");
            let time = |t: i64| if t == 0 { "无".to_string() } else { t.date_format() };
            SyncInfoView {
                no: it.no,
                url: host,
                state: status.state,
                msg: status.msg,
                progress: format!("{}/{}", status.sync_count, status.count),
                last_time: time(status.last_time),
                last_heart_time: time(status.last_heart_time),
            }
        })
        .collect()
}
