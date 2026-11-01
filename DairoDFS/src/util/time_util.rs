use std::time::{SystemTime, UNIX_EPOCH};

/// 获取系统时间戳（毫秒）
#[inline]
pub fn current_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 获取系统时间戳（微秒）
#[inline]
pub fn current_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}
