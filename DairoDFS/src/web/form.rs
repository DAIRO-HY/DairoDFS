use serde::Deserialize;
use validator::Validate;

// 表单结构体放在controller之外：hyperoute会为同文件内的请求体结构体生成multipart解析，
// 而multipart暂不支持Option字段，前端只提交urlencoded，所以这里只走serde解析

/// 登录表单
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    // 用户名
    #[validate(length(min = 1, message = "用户名不能为空"))]
    pub name: String,

    // 密码
    #[validate(length(min = 1, message = "密码不能为空"))]
    pub pwd: String,

    // 设备唯一标识
    #[validate(length(min = 1, max = 32, message = "设备标识不能为空;长度不能超过32位"))]
    pub device_id: String,

    // 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX
    #[serde(default)]
    pub client_flag: i64,

    // 客户端版本
    #[serde(default)]
    pub client_version: i64,
}

/// 安装向导:设置存储目录
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallStorage {
    // 存储目录,多个目录用换行分隔
    pub folders: String,
}

/// 安装向导:设置分布式同步
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallDistributed {
    // 同步服务器URL,多个URL用换行分隔
    #[serde(default)]
    pub sync_domains: String,
}

/// 安装向导:创建管理员
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdmin {
    // 用户名
    #[validate(length(min = 2, max = 32, message = "用户名长度必须在2到32位之间"))]
    pub name: String,

    // 密码
    #[validate(length(min = 4, max = 32, message = "密码长度必须在4到32位之间"))]
    pub pwd: String,
}

/// 用户编辑
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserEdit {
    // 用户ID,0表示新增
    #[serde(default)]
    pub id: i64,

    // 用户名
    #[validate(length(min = 2, max = 32, message = "用户名长度必须在2到32位之间"))]
    pub name: String,

    // 密码,编辑时为空表示不修改
    #[validate(length(min = 4, max = 32, message = "密码长度必须在4到32位之间"))]
    pub pwd: Option<String>,

    // 邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,

    // 用户状态 0:禁用 1:正常
    pub state: i64,
}

/// 修改密码
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ModifyPwd {
    // 旧密码
    #[validate(length(min = 1, message = "旧密码不能为空"))]
    pub old_pwd: String,

    // 新密码
    #[validate(length(min = 4, max = 32, message = "密码长度必须在4到32位之间"))]
    pub pwd: String,
}

/// 系统配置
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    // 记录同步日志
    pub open_sql_log: bool,

    // 将当前服务器设置为只读,仅作为备份使用
    pub is_read_only: bool,

    // 存储目录,多个目录用换行分隔
    #[validate(length(min = 1, message = "存储目录不能为空"))]
    pub folders: String,

    // 同步服务器URL,多个URL用换行分隔
    #[serde(default)]
    pub sync_domains: String,

    // 回收站超时(单位：天)
    #[validate(range(min = 1, message = "回收站保留天数不能小于1"))]
    pub trash_timeout: i64,

    // 删除没有被使用的文件超时设置(单位：天)
    #[validate(range(min = 1, message = "存储空间回收间隔不能小于1"))]
    pub delete_storage_timeout: i64,

    // 缩略图最大边尺寸
    #[validate(range(min = 100, max = 1024, message = "缩略图尺寸必须在100到1024之间"))]
    pub thumb_max_size: i64,

    // 忽略本机同步错误
    pub ignore_sync_error: bool,

    // 数据库备份天数
    #[validate(range(min = 1, message = "数据库备份天数不能小于1"))]
    pub db_backup_expire_day: i64,
}

/// 将空字符串转换为None(表单里没有填写的项目会提交空字符串)
pub fn none_if_empty(value: Option<String>) -> Option<String> {
    value.map(|it| it.trim().to_string()).filter(|it| !it.is_empty())
}

/// 将换行分隔的多行文本转换为列表(去掉首尾空白和空行)
pub fn split_lines(value: &str) -> Vec<String> {
    value
        .lines()
        .map(|it| it.trim().to_string())
        .filter(|it| !it.is_empty())
        .collect()
}
