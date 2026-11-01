use crate::util::time_util;
use std::sync::Mutex;

/// 记录上一次生成的id
static PRE_ID: Mutex<i64> = Mutex::new(0);

/// 生成数据库主键ID(毫秒时间戳,与老版本数据的ID规则一致)
/// 同一毫秒内多次生成时依次加1,保证唯一且递增
pub fn id() -> i64 {
    let mut pre_id = PRE_ID.lock().unwrap();
    let id = time_util::current_millis().max(*pre_id + 1);
    *pre_id = id;
    id
}
