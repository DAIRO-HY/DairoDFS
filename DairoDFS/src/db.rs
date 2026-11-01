use crate::dao::sql_log_dao;
use crate::dao::sql_log_dao::SqlLog;
use crate::service::sync::distributed_push;
use crate::util::{id_util, time_util};
use crate::web::biz_error::BizError;
use crate::{biz_error, system_config};
use sqlx_context::DbContext;

/// 开启一个事务,写数据时使用,写完之后必须调用commit
/// 始终记录写SQL:只读模式的检查和分布式同步日志都要用到
pub async fn begin() -> Result<DbContext, sqlx::Error> {
    let mut ctx = lib_db::get_context();
    ctx.enable_write_log();
    ctx.begin().await?;
    Ok(ctx)
}

/// 提交事务
/// - 只读模式下,除了登录记录(user_token)以外不允许写数据
/// - 开启了同步日志时,在同一个事务里把本次的写SQL保存到sql_log,并通知分机同步
pub async fn commit(mut ctx: DbContext) -> Result<(), BizError> {
    let logs = ctx.take_write_log();
    let config = system_config::get();
    if config.is_read_only && logs.iter().any(|it| !is_user_token_sql(&it.sql)) {
        ctx.rollback().await?;
        return Err(biz_error!("当前设置为只读模式，该操作不允许"));
    }
    let has_log = config.open_sql_log && !logs.is_empty();
    if has_log {
        //写sql_log本身不需要记录
        ctx.disable_write_log();
        for it in &logs {
            let log = SqlLog {
                id: id_util::id(),
                date: time_util::current_millis(),
                param: it.params_json(),
                sql: it.sql.clone(),
                state: 1,
                source: "0.0.0.0".to_string(),
                err: None,
            };
            sql_log_dao::insert(&mut ctx, &log).await?;
        }
    }
    ctx.commit().await?;
    if has_log {
        distributed_push::push();
    }
    Ok(())
}

/// 是否操作登录记录表的SQL
fn is_user_token_sql(sql: &str) -> bool {
    let sql = sql.to_lowercase();
    let sql = sql.trim_start();
    ["insert into user_token", "update user_token", "delete from user_token"]
        .iter()
        .any(|it| sql.starts_with(it))
}
