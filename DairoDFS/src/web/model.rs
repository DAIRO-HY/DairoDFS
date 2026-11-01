use serde::{Deserialize, Serialize};

/// 登录表单数据结构
#[derive(Deserialize, Serialize, Debug)]
pub struct ResultData<T> {
    pub code: usize,
    pub msg: String,
    pub data: T,
}
