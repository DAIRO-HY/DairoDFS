use crate::system_config;
use crate::util::time_util;
use std::sync::Mutex;

/// 服务器编号的个数,ID的末位是服务器编号
pub const NODE_COUNT: i64 = 10;

/// 记录上一次生成的id
static PRE_ID: Mutex<i64> = Mutex::new(0);

/// 生成数据库主键ID(毫秒时间戳,与老版本数据的ID规则一致)
/// 末位是本机的服务器编号,多台服务器双向同步时,不同服务器生成的ID不会重复(误差不超过9毫秒,仍然可以当作时间使用)
/// 同一毫秒内多次生成时依次往后取,保证唯一且递增
pub fn id() -> i64 {
    let node = system_config::get().node_no.rem_euclid(NODE_COUNT);
    let mut pre_id = PRE_ID.lock().unwrap();
    let id = time_util::current_millis().max(*pre_id + 1);
    let id = id + (node - id.rem_euclid(NODE_COUNT)).rem_euclid(NODE_COUNT);
    *pre_id = id;
    id
}

/// 记录同步过来的ID,之后本机生成的ID都比它大
/// 同步日志的ID用来判断修改的先后,其他服务器的时钟稍快时,本机之后的修改也一定排在它的后面
pub fn observe(id: i64) {
    let mut pre_id = PRE_ID.lock().unwrap();
    if id > *pre_id {
        *pre_id = id;
    }
}
