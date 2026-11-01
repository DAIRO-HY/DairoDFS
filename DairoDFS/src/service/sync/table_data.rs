use base64::Engine;
use serde_json::{Map, Value};
use sqlx::sqlite::{SqliteArguments, SqliteRow};
use sqlx::{Arguments, Column, Row, TypeInfo, ValueRef};
use sqlx_context::DbContext;

/// 全量同步的表(按同步顺序排列),只允许同步这些表
pub const TABLES: [&str; 6] = ["user", "user_token", "dfs_file", "dfs_file_delete", "share", "storage_file"];

/// 检查表名是否允许同步
pub fn check_table(name: &str) -> Result<&'static str, String> {
    TABLES.into_iter().find(|it| *it == name).ok_or_else(|| format!("不允许同步的表:{}", name))
}

/// 把一行数据转换成JSON对象(保留数据库里的实际类型)
pub fn row_to_json(row: &SqliteRow) -> Map<String, Value> {
    let mut map = Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        let value = match row.try_get_raw(i) {
            Ok(raw) if !raw.is_null() => match raw.type_info().name() {
                "INTEGER" => row.try_get::<i64, _>(i).map(Value::from).unwrap_or(Value::Null),
                "REAL" => row.try_get::<f64, _>(i).map(Value::from).unwrap_or(Value::Null),
                "BLOB" => row
                    .try_get::<Vec<u8>, _>(i)
                    .map(|it| Value::from(base64::engine::general_purpose::STANDARD.encode(it)))
                    .unwrap_or(Value::Null),
                _ => row.try_get::<String, _>(i).map(Value::from).unwrap_or(Value::Null),
            },
            _ => Value::Null,
        };
        map.insert(column.name().to_string(), value);
    }
    map
}

/// 查询数据并转换成JSON
pub async fn select(ctx: &mut DbContext, sql: &str, params: &[Value]) -> Result<Vec<Map<String, Value>>, sqlx::Error> {
    let rows = sqlx::query_with(sql, to_arguments(params)).fetch_all(&mut *ctx).await?;
    Ok(rows.iter().map(row_to_json).collect())
}

/// 执行SQL,参数为JSON数组(同步日志里保存的参数)
pub async fn execute(ctx: &mut DbContext, sql: &str, params: &[Value]) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query_with(sql, to_arguments(params)).execute(&mut *ctx).await?.rows_affected())
}

/// 把JSON参数转换成SQL参数
fn to_arguments(params: &[Value]) -> SqliteArguments<'static> {
    let mut args = SqliteArguments::default();
    for it in params {
        let _ = match it {
            Value::Null => args.add(None::<String>),
            Value::Bool(b) => args.add(*b as i64),
            Value::Number(n) => match n.as_i64() {
                Some(i) => args.add(i),
                None => args.add(n.as_f64().unwrap_or(0.0)),
            },
            Value::String(s) => args.add(s.clone()),
            other => args.add(other.to_string()),
        };
    }
    args
}

/// 插入一行数据
pub async fn insert(ctx: &mut DbContext, table: &str, row: &Map<String, Value>) -> Result<(), sqlx::Error> {
    let keys: Vec<&str> = row.keys().map(|it| it.as_str()).collect();
    let sql = format!(
        "insert into {} ({}) values ({})",
        table,
        keys.iter().map(|it| format!("\"{}\"", it.replace('"', ""))).collect::<Vec<_>>().join(","),
        vec!["?"; keys.len()].join(",")
    );
    let values: Vec<Value> = row.values().cloned().collect();
    execute(ctx, &sql, &values).await?;
    Ok(())
}
