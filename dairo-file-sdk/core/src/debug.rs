//! 调试用的运行状态统计,只在内存里计数,从本次SDK启动开始

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// 最多保存的事件数
const MAX_EVENT_COUNT: usize = 100;

/// 一条服务事件
#[derive(Debug, Clone)]
pub struct DebugEvent {
    /// 时间(毫秒)
    pub time: i64,
    pub msg: String,
}

/// 运行状态
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    /// SDK启动时间(毫秒)
    pub sdk_start_time: i64,

    /// 本机HTTP服务是否标记为运行中
    pub server_running: bool,

    /// 本机HTTP服务监听的端口,没有运行时为None
    pub server_port: Option<u16>,

    /// 本机HTTP服务这次启动的时间(毫秒),没有运行时为None
    pub server_start_time: Option<i64>,

    /// 本机HTTP服务启动的次数(包括第一次启动、自动重启、手动重启)
    pub server_start_count: u64,

    /// 手动重启的次数
    pub server_restart_count: u64,

    /// 本机HTTP服务收到的请求数
    pub server_request_count: u64,

    /// 当前连接数
    pub server_connection_count: u64,

    /// 本机HTTP服务返回的数据量(字节)
    pub server_bytes_sent: u64,

    /// 返回2xx、4xx、5xx和其他状态码的请求数
    pub server_status_2xx: u64,
    pub server_status_4xx: u64,
    pub server_status_5xx: u64,
    pub server_status_other: u64,

    /// 接收连接失败的次数和最后一次的错误
    pub accept_error_count: u64,
    pub last_accept_error: Option<String>,

    /// 发起的网络请求数(下载和上传)
    pub net_request_count: u64,

    /// 网络请求失败的次数(连接失败、读取中断、服务器返回错误等)
    pub net_error_count: u64,

    /// 下载的数据量(字节)
    pub net_bytes_received: u64,

    /// 上传的数据量(字节)
    pub net_bytes_sent: u64,

    /// 正在进行的下载任务数
    pub downloading_count: u64,

    /// 正在进行的上传任务数
    pub uploading_count: u64,

    /// 最近的服务事件,按时间倒序
    pub events: Vec<DebugEvent>,
}

/// 计数器
#[derive(Default)]
pub(crate) struct Stats {
    pub server_start_count: AtomicU64,
    pub server_restart_count: AtomicU64,
    pub server_request_count: AtomicU64,
    pub server_connection_count: AtomicU64,
    pub server_bytes_sent: AtomicU64,
    pub server_status_2xx: AtomicU64,
    pub server_status_4xx: AtomicU64,
    pub server_status_5xx: AtomicU64,
    pub server_status_other: AtomicU64,
    pub accept_error_count: AtomicU64,
    pub last_accept_error: Mutex<Option<String>>,
    pub net_request_count: AtomicU64,
    pub net_error_count: AtomicU64,
    pub net_bytes_received: AtomicU64,
    pub net_bytes_sent: AtomicU64,
    events: Mutex<VecDeque<DebugEvent>>,
}

impl Stats {
    pub fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(counter: &AtomicU64) -> u64 {
        counter.load(Ordering::Relaxed)
    }

    /// 记录本机HTTP服务返回的状态码
    pub fn status(&self, status: u16) {
        let counter = match status {
            200..=299 => &self.server_status_2xx,
            400..=499 => &self.server_status_4xx,
            500..=599 => &self.server_status_5xx,
            _ => &self.server_status_other,
        };
        Self::add(counter, 1);
    }

    /// 记录一条服务事件
    pub fn event(&self, msg: impl Into<String>) {
        let mut events = self.events.lock().unwrap();
        events.push_back(DebugEvent { time: crate::now_millis(), msg: msg.into() });
        while events.len() > MAX_EVENT_COUNT {
            events.pop_front();
        }
    }

    /// 最近的事件,按时间倒序
    pub fn events(&self) -> Vec<DebugEvent> {
        self.events.lock().unwrap().iter().rev().cloned().collect()
    }
}

/// 自检结果
#[derive(Debug, Clone, Default)]
pub struct SelfCheckResult {
    /// 是否正常:连接成功并且收到了ping的200响应
    pub ok: bool,

    /// 服务是否标记为运行中(没有运行时不会自动启动)
    pub running: bool,

    /// 检查的端口
    pub port: Option<u16>,

    /// TCP是否连接成功
    pub connected: bool,

    /// HTTP状态码,没有收到响应时为None
    pub status: Option<u16>,

    /// 耗时(毫秒)
    pub elapsed_ms: u64,

    /// 失败的原因
    pub error: Option<String>,
}
