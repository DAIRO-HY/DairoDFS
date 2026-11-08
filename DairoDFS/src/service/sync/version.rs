//! 同一条记录在多台服务器上同时修改时,以最后修改的为准
//!
//! 每次修改记录下这组字段最后一次被修改的同步日志ID(包含时间和服务器编号,越大越新)
//! 执行其他服务器同步过来的修改时,本机这组字段已经有更新的修改,就不执行
//! 一条语句同时修改了多组字段,并且只有一部分比本机新时,其他组的字段保持本机现在的值
//!
//! 同步日志里的每一条修改语句都要在下面登记,没有登记的修改语句仍然直接执行(两边同时修改时可能不一致)

use serde_json::{Map, Value};
use sqlx_context::DbContext;

use crate::service::sync::table_data;

/// 一条修改语句
pub struct Stmt {

    // 语句(和生成的DAO里的一样,比较时忽略空格和大小写)
    pub sql: &'static str,

    // 表名
    pub table: &'static str,

    // 记录ID是第几个参数
    pub id: usize,

    // 修改的字段,以及它的值是第几个参数(None:固定值)
    pub columns: &'static [(&'static str, Option<usize>)],
}

/// 需要按最后修改为准的表,以及每张表的字段组(同一组的字段总是一起修改)
pub const GROUPS: [(&str, &[(&str, &[&str])]); 3] = [
    (
        "dfs_file",
        &[
            ("pos", &["parentId", "name"]),
            ("deleteDate", &["deleteDate"]),
            ("contentType", &["contentType"]),
            ("property", &["property"]),
            ("state", &["state", "stateMsg"]),
            ("isHistory", &["isHistory"]),
        ],
    ),
    (
        "user",
        &[
            ("name", &["name"]),
            ("pwd", &["pwd"]),
            ("email", &["email"]),
            ("urlPath", &["urlPath"]),
            ("apiToken", &["apiToken"]),
            ("encryptionKey", &["encryptionKey"]),
            ("state", &["state"]),
            ("nickname", &["nickname"]),
            ("avatar", &["avatar"]),
            ("handleFile", &["handleFile"]),
        ],
    ),
    ("sub_account_grant", &[("perms", &["perms"])]),
];

/// 登记的修改语句
pub const STMTS: [Stmt; 17] = [
    Stmt { sql: "update dfs_file set isHistory = 1 where id = ?;", table: "dfs_file", id: 0, columns: &[("isHistory", None)] },
    Stmt { sql: "update dfs_file set deleteDate = ? where id = ?;", table: "dfs_file", id: 1, columns: &[("deleteDate", Some(0))] },
    Stmt {
        sql: "update dfs_file set contentType = ? where id = ? and storageId > 0;",
        table: "dfs_file",
        id: 1,
        columns: &[("contentType", Some(0))],
    },
    Stmt {
        sql: "update dfs_file set parentId = ?, name = ? where id = ?;",
        table: "dfs_file",
        id: 2,
        columns: &[("parentId", Some(0)), ("name", Some(1))],
    },
    Stmt { sql: "update dfs_file set property = ? where id = ?;", table: "dfs_file", id: 1, columns: &[("property", Some(0))] },
    Stmt {
        sql: "update dfs_file set state = ?, stateMsg = ? where id = ?;",
        table: "dfs_file",
        id: 2,
        columns: &[("state", Some(0)), ("stateMsg", Some(1))],
    },
    Stmt {
        sql: "update user set name = ?, nickname = ?, state = ? where id = ?;",
        table: "user",
        id: 3,
        columns: &[("name", Some(0)), ("nickname", Some(1)), ("state", Some(2))],
    },
    Stmt {
        sql: "update user set state = ?, name = ?, pwd = ?, apiToken = ?, urlPath = ? where id = ?;",
        table: "user",
        id: 5,
        columns: &[("state", Some(0)), ("name", Some(1)), ("pwd", Some(2)), ("apiToken", Some(3)), ("urlPath", Some(4))],
    },
    Stmt {
        sql: "update user set name = ?, email = ?, state = ? where id = ?;",
        table: "user",
        id: 3,
        columns: &[("name", Some(0)), ("email", Some(1)), ("state", Some(2))],
    },
    Stmt { sql: "update user set urlPath = ? where id = ?;", table: "user", id: 1, columns: &[("urlPath", Some(0))] },
    Stmt { sql: "update user set apiToken = ? where id = ?;", table: "user", id: 1, columns: &[("apiToken", Some(0))] },
    Stmt { sql: "update user set encryptionKey = ? where id = ?;", table: "user", id: 1, columns: &[("encryptionKey", Some(0))] },
    Stmt { sql: "update user set pwd = ? where id = ?;", table: "user", id: 1, columns: &[("pwd", Some(0))] },
    Stmt { sql: "update user set nickname = ? where id = ?;", table: "user", id: 1, columns: &[("nickname", Some(0))] },
    Stmt { sql: "update user set avatar = ? where id = ?;", table: "user", id: 1, columns: &[("avatar", Some(0))] },
    Stmt { sql: "update user set handleFile = ? where id = ?;", table: "user", id: 1, columns: &[("handleFile", Some(0))] },
    Stmt { sql: "update sub_account_grant set perms = ? where id = ?;", table: "sub_account_grant", id: 1, columns: &[("perms", Some(0))] },
];

/// 去掉空格、结尾的分号并转换成小写,用来比较语句
pub fn normalize(sql: &str) -> String {
    sql.chars()
        .filter(|it| !it.is_whitespace())
        .collect::<String>()
        .trim_end_matches(';')
        .to_lowercase()
}

/// 找到登记的修改语句
pub fn find(sql: &str) -> Option<&'static Stmt> {
    let sql = normalize(sql);
    STMTS.iter().find(|it| normalize(it.sql) == sql)
}

/// 表的字段组
fn groups(table: &str) -> &'static [(&'static str, &'static [&'static str])] {
    GROUPS.iter().find(|it| it.0 == table).map(|it| it.1).unwrap_or(&[])
}

/// 是否需要按最后修改为准的表
pub fn is_versioned(table: &str) -> bool {
    GROUPS.iter().any(|it| it.0 == table)
}

/// 语句修改了哪些字段组
fn touched_groups(stmt: &Stmt) -> Vec<&'static str> {
    groups(stmt.table)
        .iter()
        .filter(|(_, columns)| columns.iter().any(|c| stmt.columns.iter().any(|it| it.0 == *c)))
        .map(|it| it.0)
        .collect()
}

/// 记录ID
fn row_id(stmt: &Stmt, params: &[Value]) -> Option<i64> {
    params.get(stmt.id).and_then(|it| it.as_i64())
}

/// 获取一条记录各个字段组的修改时间
pub async fn select(ctx: &mut DbContext, table: &str, id: i64) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = table_data::select(ctx, "select grp, ver from sync_version where tb = ? and rowId = ?", &[table.into(), id.into()]).await?;
    Ok(rows
        .iter()
        .filter_map(|it| Some((it.get("grp")?.as_str()?.to_string(), it.get("ver")?.as_i64()?)))
        .collect())
}

/// 记录字段组的修改时间(只会往新的改)
pub async fn save(ctx: &mut DbContext, table: &str, id: i64, group: &str, ver: i64) -> Result<(), sqlx::Error> {
    table_data::execute(
        ctx,
        "insert into sync_version(tb, rowId, grp, ver) values (?, ?, ?, ?) on conflict(tb, rowId, grp) do update set ver = max(ver, excluded.ver)",
        &[table.into(), id.into(), group.into(), ver.into()],
    )
    .await?;
    Ok(())
}

/// 本机修改了数据,记录修改时间(在写数据的同一个事务里执行)
/// - log_id 这条修改的同步日志ID
pub async fn record_local(ctx: &mut DbContext, sql: &str, params: &[Value], log_id: i64) -> Result<(), sqlx::Error> {
    let Some(stmt) = find(sql) else {
        return Ok(());
    };
    let Some(id) = row_id(stmt, params) else {
        return Ok(());
    };
    for group in touched_groups(stmt) {
        save(ctx, stmt.table, id, group, log_id).await?;
    }
    Ok(())
}

/// 执行同步过来的修改之前,判断哪些字段需要修改
pub enum Decision {

    // 本机有更新的修改(或者记录已经被删除),不执行
    Skip,

    // 执行,参数里本机更新的字段已经换成了本机现在的值
    Execute { params: Vec<Value>, groups: Vec<&'static str> },
}

/// 判断同步过来的修改哪些需要执行
/// - log_id 同步日志的ID
pub async fn decide(ctx: &mut DbContext, stmt: &Stmt, mut params: Vec<Value>, log_id: i64) -> Result<Decision, sqlx::Error> {
    let Some(id) = row_id(stmt, &params) else {
        return Ok(Decision::Execute { params, groups: Vec::new() });
    };
    let exists = table_data::select(ctx, &format!("select * from {} where id = ?", stmt.table), &[id.into()]).await?;
    let Some(row) = exists.first() else {
        //记录已经被删除,删除优先
        return Ok(Decision::Skip);
    };
    let versions = select(ctx, stmt.table, id).await?;
    let ver = |group: &str| versions.iter().find(|it| it.0 == group).map(|it| it.1).unwrap_or(0);
    let (newer, older): (Vec<_>, Vec<_>) = touched_groups(stmt).into_iter().partition(|it| log_id > ver(it));
    if newer.is_empty() {
        return Ok(Decision::Skip);
    }

    //本机更新的字段组,保持本机现在的值
    for group in older {
        let columns = groups(stmt.table).iter().find(|it| it.0 == group).map(|it| it.1).unwrap_or(&[]);
        for (column, index) in stmt.columns {
            if !columns.contains(column) {
                continue;
            }
            match index {
                Some(index) => params[*index] = row.get(*column).cloned().unwrap_or(Value::Null),
                //固定值的语句只修改一组字段,不会走到这里
                None => return Ok(Decision::Skip),
            }
        }
    }
    Ok(Decision::Execute { params, groups: newer })
}

/// 记录执行之后的修改时间
pub async fn save_groups(ctx: &mut DbContext, stmt: &Stmt, params: &[Value], groups: &[&str], log_id: i64) -> Result<(), sqlx::Error> {
    let Some(id) = row_id(stmt, params) else {
        return Ok(());
    };
    for group in groups {
        save(ctx, stmt.table, id, group, log_id).await?;
    }
    Ok(())
}

/// 全量同步时,本机已经有这条记录并且和主机不一致,按字段组以最后修改的为准合并
/// 不一致的字段两边都没有修改时间(不属于任何字段组,或者老版本的数据就不一致)时无法判断,使用主机的值
/// - return (合并之后的记录, 无法判断以哪边为准的字段)
pub fn merge_row(
    table: &str,
    local: &Map<String, Value>,
    master: &Map<String, Value>,
    master_versions: &[(String, i64)],
    local_versions: &[(String, i64)],
) -> (Map<String, Value>, Option<String>) {
    let ver = |versions: &[(String, i64)], group: &str| versions.iter().find(|it| it.0 == group).map(|it| it.1).unwrap_or(0);
    let mut merged = local.clone();
    let mut conflict = None;
    for (key, value) in master {
        if local.get(key) == Some(value) {
            continue;
        }
        if table == "dfs_file" && key == "isHistory" {
            //历史版本只会从0变成1(冲突时把旧的文件设置为历史版本,这个修改没有同步日志),取两边的较大值
            let max = value.as_i64().unwrap_or(0).max(local.get(key).and_then(|it| it.as_i64()).unwrap_or(0));
            merged.insert(key.clone(), Value::from(max));
            continue;
        }
        let group = groups(table).iter().find(|it| it.1.contains(&key.as_str())).map(|it| it.0);
        let (master_ver, local_ver) = match group {
            Some(group) => (ver(master_versions, group), ver(local_versions, group)),
            None => (0, 0),
        };
        if master_ver > local_ver {
            merged.insert(key.clone(), value.clone());
        } else if master_ver == 0 && local_ver == 0 {
            if conflict.is_none() {
                conflict = Some(key.clone());
            }
            merged.insert(key.clone(), value.clone());
        }
    }
    (merged, conflict)
}

/// 删除指定时间之前的修改时间记录(和同步日志的保留时间一样,超过这个时间的修改不会再同步过来)
pub async fn delete_before(ctx: &mut DbContext, time: i64) -> Result<(), sqlx::Error> {
    table_data::execute(ctx, "delete from sync_version where ver < ?", &[time.into()]).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 同步的表里所有的修改语句都要登记,字段组要完整
    #[test]
    fn all_update_registered() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dsl");
        let param = regex_lite_replace;
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
            for block in text.split("-- ------------------------------") {
                let sql = block
                    .lines()
                    .filter(|it| !it.trim_start().starts_with("--"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let sql = sql.trim();
                if !sql.to_lowercase().starts_with("update") {
                    continue;
                }
                let table = sql.split_whitespace().nth(1).unwrap();
                if !crate::service::sync::table_data::TABLES.contains(&table) {
                    //不同步的表
                    continue;
                }
                let sql = param(sql);
                let stmt = find(&sql).unwrap_or_else(|| panic!("修改语句没有登记:{}", sql));
                assert!(is_versioned(stmt.table), "表没有字段组:{}", stmt.table);
                checked += 1;
            }
        }
        assert!(checked >= STMTS.len());
        for stmt in &STMTS {
            //修改的字段必须属于某一个字段组,并且一组字段要一起修改
            for (column, _) in stmt.columns {
                assert!(groups(stmt.table).iter().any(|it| it.1.contains(column)), "{} 字段没有分组:{}", stmt.sql, column);
            }
            for group in touched_groups(stmt) {
                let columns = groups(stmt.table).iter().find(|it| it.0 == group).unwrap().1;
                for column in columns {
                    assert!(stmt.columns.iter().any(|it| it.0 == *column), "{} 只修改了字段组{}的一部分", stmt.sql, group);
                }
            }
            //固定值只能出现在只修改一组字段的语句里
            if stmt.columns.iter().any(|it| it.1.is_none()) {
                assert_eq!(touched_groups(stmt).len(), 1, "{}", stmt.sql);
            }
            //参数个数
            let count = stmt.sql.matches('?').count();
            assert!(stmt.id < count);
            for (_, index) in stmt.columns {
                if let Some(index) = index {
                    assert!(*index < count);
                }
            }
        }
    }

    /// DSL里的 :name 参数换成 ?
    fn regex_lite_replace(sql: &str) -> String {
        let mut out = String::new();
        let mut chars = sql.chars().peekable();
        while let Some(c) = chars.next() {
            if c == ':' && chars.peek().is_some_and(|it| it.is_ascii_alphabetic() || *it == '_') {
                while chars.peek().is_some_and(|it| it.is_ascii_alphanumeric() || *it == '_') {
                    chars.next();
                }
                out.push('?');
            } else {
                out.push(c);
            }
        }
        out
    }
}
