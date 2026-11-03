use crate::dao::dfs_file_dao::DfsFile;
use crate::dao::operate_log_dao;
use crate::dao::operate_log_dao::OperateLog;
use crate::service::dfs_file_service;
use crate::system_config;
use crate::util::{id_util, time_util};
use crate::web::filter::auth::LoginUser;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc;

/// 操作类型
pub mod op {
    pub const LOGIN: i64 = 1;
    pub const LOGOUT: i64 = 2;
    pub const MODIFY_PWD: i64 = 3;
    pub const UPLOAD: i64 = 10;
    pub const CREATE_FOLDER: i64 = 11;
    pub const RENAME: i64 = 12;
    pub const MOVE: i64 = 13;
    pub const COPY: i64 = 14;
    pub const DELETE: i64 = 15;
    pub const RECOVER: i64 = 16;
    pub const PURGE: i64 = 17;
    pub const CONTENT_TYPE: i64 = 18;
    pub const CAPTURE_TIME: i64 = 19;
    pub const OPEN: i64 = 20;
    pub const SHARE: i64 = 30;
    pub const SHARE_DELETE: i64 = 31;
    pub const SAVE_TO: i64 = 32;
    pub const SUB_ACCOUNT_ADD: i64 = 40;
    pub const SUB_ACCOUNT_EDIT: i64 = 41;
    pub const SUB_ACCOUNT_DELETE: i64 = 42;

    /// 所有操作类型和名称,用于显示和筛选
    pub const ALL: [(i64, &str); 20] = [
        (LOGIN, "登录"),
        (LOGOUT, "退出登录"),
        (MODIFY_PWD, "修改密码"),
        (UPLOAD, "上传"),
        (CREATE_FOLDER, "新建文件夹"),
        (RENAME, "重命名"),
        (MOVE, "移动"),
        (COPY, "复制"),
        (DELETE, "删除"),
        (RECOVER, "从回收站恢复"),
        (PURGE, "彻底删除"),
        (CONTENT_TYPE, "修改文件类型"),
        (CAPTURE_TIME, "修改拍摄时间"),
        (OPEN, "打开/下载"),
        (SHARE, "创建分享"),
        (SHARE_DELETE, "取消分享"),
        (SAVE_TO, "转存分享"),
        (SUB_ACCOUNT_ADD, "添加子账户"),
        (SUB_ACCOUNT_EDIT, "修改子账户"),
        (SUB_ACCOUNT_DELETE, "删除子账户"),
    ];

    /// 操作类型的名称
    pub fn name(op_type: i64) -> &'static str {
        ALL.iter().find(|it| it.0 == op_type).map(|it| it.1).unwrap_or("未知")
    }
}

/// 同一个人在这段时间内重复打开同一个文件(例如播放器拖动进度)只记录一次
const OPEN_DEDUP_MILLIS: i64 = 10 * 60 * 1000;

/// 一次最多写入的日志条数
const BATCH_SIZE: usize = 500;

/// 一天的毫秒数
const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// 日志队列,日志在后台批量写入,不影响接口的响应速度
static SENDER: OnceLock<mpsc::UnboundedSender<OperateLog>> = OnceLock::new();

/// 最近打开过的文件:(用户ID, 文件ID) -> (主账户ID, 打开时间)
static RECENT_OPEN: LazyLock<Mutex<HashMap<(i64, i64), (i64, i64)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 启动日志写入任务和定时清理任务
pub fn start() {
    let (sender, receiver) = mpsc::unbounded_channel();
    if SENDER.set(sender).is_err() {
        return;
    }
    tokio::spawn(write_loop(receiver));
    tokio::spawn(async {
        loop {
            clean().await;
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });
}

/// 记录一条操作日志
/// - file_id 操作的文件ID
/// - path 操作对象的路径
/// - detail 其他信息,例如移动的目标路径
pub fn add(login: &LoginUser, op_type: i64, file_id: Option<i64>, path: Option<String>, detail: Option<Value>) {
    add_raw(login.id, &login.name, login.owner_id, &login.ip, login.client_flag, op_type, file_id, path, detail);
}

/// 记录一条文件操作日志
/// 文件被修改、移动、删除等之后,之前打开的去重记录失效,之后再打开时重新记录;操作的是文件夹时,该账户下所有文件的去重记录都失效
pub fn add_file(login: &LoginUser, op_type: i64, file: &DfsFile, path: String, detail: Option<Value>) {
    {
        let mut recent = RECENT_OPEN.lock().unwrap();
        if file.storageId == 0 {
            recent.retain(|_, it| it.0 != login.owner_id);
        } else {
            recent.retain(|key, _| key.1 != file.id);
        }
    }
    add(login, op_type, Some(file.id), Some(path), detail);
}

/// 记录打开或下载文件,同一个人10分钟内重复打开同一个文件只记录一次
/// - detail 其他信息,例如打开的是哪个附属文件
pub async fn add_open(login: &LoginUser, file: &DfsFile, detail: Option<Value>) {
    let now = time_util::current_millis();
    {
        let mut recent = RECENT_OPEN.lock().unwrap();
        let key = (login.id, file.id);
        if recent.get(&key).is_some_and(|it| now - it.1 < OPEN_DEDUP_MILLIS) {
            return;
        }
        if recent.len() > 10000 {
            recent.retain(|_, it| now - it.1 < OPEN_DEDUP_MILLIS);
        }
        recent.insert(key, (login.owner_id, now));
    }
    let path = dfs_file_service::get_path_by_id(&mut lib_db::get_context(), file.id).await.ok();
    add(login, op::OPEN, Some(file.id), path, detail);
}

/// 记录一条操作日志(没有登录信息时使用,例如登录)
#[allow(clippy::too_many_arguments)]
pub fn add_raw(
    user_id: i64,
    user_name: &str,
    owner_id: i64,
    ip: &str,
    client_flag: i64,
    op_type: i64,
    file_id: Option<i64>,
    path: Option<String>,
    detail: Option<Value>,
) {
    let Some(sender) = SENDER.get() else {
        return;
    };
    let log = OperateLog {
        id: id_util::id(),
        date: time_util::current_millis(),
        userId: user_id,
        userName: user_name.to_string(),
        ownerId: owner_id,
        opType: op_type,
        fileId: file_id,
        path,
        detail: detail.map(|it| it.to_string()),
        ip: ip.to_string(),
        clientFlag: client_flag,
    };
    let _ = sender.send(log);
}

/// 批量写入日志
/// 日志只保存在本机,不经过db::commit:不记录同步日志,只读模式下也能写入
async fn write_loop(mut receiver: mpsc::UnboundedReceiver<OperateLog>) {
    let mut list = Vec::new();
    while receiver.recv_many(&mut list, BATCH_SIZE).await > 0 {
        if let Err(e) = write(&list).await {
            eprintln!("-->写入操作日志失败:{}", e);
        }
        list.clear();
    }
}

async fn write(list: &[OperateLog]) -> Result<(), sqlx::Error> {
    let mut ctx = lib_db::get_context();
    ctx.begin_immediate().await?;
    for it in list {
        operate_log_dao::insert(&mut ctx, it).await?;
    }
    ctx.commit().await?;
    Ok(())
}

/// 删除超过保存天数的日志
async fn clean() {
    let days = system_config::get().operate_log_days.max(1);
    let time = time_util::current_millis() - days * DAY_MILLIS;
    if let Err(e) = operate_log_dao::delete_before(&mut lib_db::get_context(), time).await {
        eprintln!("-->清理操作日志失败:{}", e);
    }
}
