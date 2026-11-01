use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tokio::sync::Notify;

/// 长连接心跳间隔时间(秒)
pub const KEEP_ALIVE_TIME: u64 = 120;

/// 有新的同步日志时,唤醒所有等待中的分机长连接
pub static NOTIFY: Notify = Notify::const_new();

/// 分机端票据 -> 当前长连接的编号,同一个分机重新连接时,旧的连接自动结束
static LISTENERS: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 通知分机端同步
pub fn push() {
    NOTIFY.notify_waiters();
}

/// 登记一个新的长连接,返回它的编号
pub fn register(client_token: &str) -> u64 {
    let no = {
        let mut listeners = LISTENERS.lock().unwrap();
        let no = listeners.get(client_token).copied().unwrap_or(0) + 1;
        listeners.insert(client_token.to_string(), no);
        no
    };

    //唤醒旧的连接,让它结束
    push();
    no
}

/// 是否还是该分机最新的连接
pub fn is_current(client_token: &str, no: u64) -> bool {
    LISTENERS.lock().unwrap().get(client_token) == Some(&no)
}
