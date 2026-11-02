use crate::dao::operate_log_dao;
use crate::dao::operate_log_dao::{OperateLogCountQuery, OperateLogQuery};
use crate::extension::number::NumberExtension;
use crate::service::operate_log_service::op;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use hyperoute_core::Ext;
use hyperoute_macro::GET;

/// 一页最多显示的日志条数
const MAX_LIMIT: i64 = 500;

/// 查询操作日志(按时间倒序)
/// 管理员可以查看所有日志,主账户可以查看自己和子账户的日志,子账户只能查看自己的日志
/// - owner_id 主账户ID,仅管理员使用
/// - user_id 操作人ID
/// - op_type 操作类型
/// - start,end 时间范围(毫秒)
/// - keyword 路径关键字
/// - offset,limit 分页
#[GET("/api/operate_log/list")]
#[allow(clippy::too_many_arguments)]
pub async fn list(
    login: Ext<LoginUser>,
    owner_id: Option<i64>,
    user_id: Option<i64>,
    op_type: Option<i64>,
    start: Option<i64>,
    end: Option<i64>,
    keyword: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<model::LogList, BizError> {
    let (owner_id, user_id) = scope(&login, owner_id, user_id);
    let keyword = keyword.map(|it| it.trim().to_string()).filter(|it| !it.is_empty());
    let mut ctx = lib_db::get_context();
    let count_query = OperateLogCountQuery {
        owner_id,
        user_id,
        op_type,
        start,
        end,
        keyword: keyword.clone(),
    };
    let total = operate_log_dao::count_list(&mut ctx, count_query).await?;
    let query = OperateLogQuery {
        owner_id,
        user_id,
        op_type,
        start,
        end,
        keyword,
        limit: limit.unwrap_or(50).clamp(1, MAX_LIMIT),
        offset: offset.unwrap_or(0).max(0),
    };
    let list = operate_log_dao::select_list(&mut ctx, query)
        .await?
        .into_iter()
        .map(|it| model::LogItem {
            id: it.id,
            date: it.date,
            date_str: it.date.date_format(),
            user_id: it.userId,
            user_name: it.userName,
            is_sub: it.userId != it.ownerId,
            op_type: it.opType,
            op_name: op::name(it.opType).to_string(),
            file_id: it.fileId,
            path: it.path.unwrap_or_default(),
            detail: it.detail.and_then(|it| serde_json::from_str(&it).ok()),
            ip: it.ip,
            client_flag: it.clientFlag,
        })
        .collect();
    Ok(model::LogList { total, list })
}

/// 操作类型列表,用于筛选
#[GET("/api/operate_log/types")]
pub async fn types() -> Vec<model::LogType> {
    op::ALL
        .iter()
        .map(|it| model::LogType {
            value: it.0,
            name: it.1.to_string(),
        })
        .collect()
}

/// 日志里出现过的操作人,用于筛选(包括已经删除的子账户)
#[GET("/api/operate_log/users")]
pub async fn users(login: Ext<LoginUser>) -> Result<Vec<model::LogUser>, BizError> {
    if login.is_sub {
        return Ok(vec![model::LogUser {
            id: login.id,
            name: login.name.clone(),
        }]);
    }
    let owner_id = if login.is_admin { None } else { Some(login.id) };
    let list = operate_log_dao::select_users(&mut lib_db::get_context(), owner_id)
        .await?
        .into_iter()
        .filter_map(|it| {
            Some(model::LogUser {
                id: it.userId?,
                name: it.userName,
            })
        })
        .collect();
    Ok(list)
}

/// 按登录用户限制查询范围
/// - return 主账户ID、操作人ID
fn scope(login: &LoginUser, owner_id: Option<i64>, user_id: Option<i64>) -> (Option<i64>, Option<i64>) {
    if login.is_sub {
        return (Some(login.owner_id), Some(login.id));
    }
    if login.is_admin {
        return (owner_id, user_id);
    }
    (Some(login.id), user_id)
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LogList {

        // 总条数
        pub total: i64,

        // 日志
        pub list: Vec<LogItem>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LogItem {

        // 日志ID
        pub id: i64,

        // 操作时间(毫秒)
        pub date: i64,

        // 操作时间(服务器时区)
        pub date_str: String,

        // 操作人ID
        pub user_id: i64,

        // 操作人的用户名(操作时)
        pub user_name: String,

        // 操作人是否子账户
        pub is_sub: bool,

        // 操作类型
        pub op_type: i64,

        // 操作类型名称
        pub op_name: String,

        // 操作的文件ID
        pub file_id: Option<i64>,

        // 操作对象的路径(操作时)
        pub path: String,

        // 其他信息
        pub detail: Option<serde_json::Value>,

        // 客户端IP地址
        pub ip: String,

        // 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX -1:API票据
        pub client_flag: i64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LogType {

        // 操作类型
        pub value: i64,

        // 名称
        pub name: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LogUser {

        // 用户ID
        pub id: i64,

        // 用户名
        pub name: String,
    }
}
