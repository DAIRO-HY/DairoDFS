//! 合并记录:多台服务器各自创建的同名文件夹、同样内容的存储文件合并之后,被合并掉的ID对应到保留下来的ID
//! 合并时总是保留ID小的(先创建的),每台服务器的合并结果都一样

use crate::service::sync::table_data;
use crate::util::time_util;
use sqlx_context::DbContext;

/// 获取合并之后的ID,没有被合并时返回原来的ID
pub async fn resolve(ctx: &mut DbContext, id: i64) -> Result<i64, sqlx::Error> {
    let mut id = id;

    //被合并的记录又被合并时,一直找到最后保留下来的(限制次数,避免数据异常时死循环)
    for _ in 0..16 {
        let rows = table_data::select(ctx, "select targetId from sync_alias where id = ?", &[id.into()]).await?;
        match rows.first().and_then(|it| it.get("targetId")).and_then(|it| it.as_i64()) {
            Some(target) => id = target,
            None => break,
        }
    }
    Ok(id)
}

/// 记录合并
pub async fn add(ctx: &mut DbContext, id: i64, target_id: i64, table: &str) -> Result<(), sqlx::Error> {
    table_data::execute(
        ctx,
        "insert or replace into sync_alias(id, targetId, tb, date) values (?, ?, ?, ?)",
        &[id.into(), target_id.into(), table.into(), time_util::current_millis().into()],
    )
    .await?;
    Ok(())
}
