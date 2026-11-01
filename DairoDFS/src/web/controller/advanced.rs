use crate::dao::storage_file_dao;
use crate::extension::number::NumberExtension;
use crate::service::sync::table_data;
use crate::service::{file_handle_service, recycle_service, sync};
use crate::util::{db_backup_util, file_util, time_util};
use crate::web::biz_error::BizError;
use crate::{biz_error, system_config};
use hyperoute_macro::{GET, POST};
use serde_json::{json, Value};
use sqlx::{Column, Executor, Row};
use std::sync::atomic::Ordering;
use std::sync::Mutex;

/// 执行SQL时最多返回的数据量
const MAX_SELECT_LIMIT: i64 = 10000;

/// 立即回收时,只回收这个时间之前创建的存储文件,避免正在上传的文件被回收
const RECYCLE_NOW_SAFE_MILLIS: i64 = 10 * 60 * 1000;

/// 验证文件完整的状态
static VALIDATE: Mutex<model::ValidateState> = Mutex::new(model::ValidateState {
    is_running: false,
    is_stop: false,
    total: 0,
    current: 0,
    errors: Vec::new(),
});

/// 页面初始化数据
#[GET("/api/advanced/info")]
pub async fn info() -> model::AdvancedInfo {
    let mut storage_state = "磁盘使用（已使用/总大小）：".to_string();
    for it in &system_config::get().save_folder_list {
        match (fs4::total_space(it), fs4::available_space(it)) {
            (Ok(total), Ok(available)) => {
                storage_state += &format!("{}({}/{});", it, (total - available).data_size(), total.data_size());
            }
            (Err(e), _) | (_, Err(e)) => storage_state += &format!("{}({});", it, e),
        }
    }
    model::AdvancedInfo {
        file_handling: if file_handle_service::IS_HANDLING.load(Ordering::Relaxed) { "正在处理" } else { "空闲中" }.to_string(),
        recycle_storage_timer_state: recycle_service::state_text(),
        storage_state,
    }
}

/// 执行SQL,查询语句返回查询结果,其他语句返回影响的行数
/// 直接操作数据库,不记录同步日志
#[POST("/api/advanced/exec_sql")]
pub async fn exec_sql(sql: String) -> Result<Value, BizError> {
    let sql = sql.trim().trim_end_matches(';').to_string();
    let pool = lib_db::get();
    if !sql.to_lowercase().starts_with("select") {
        let rs = sqlx::raw_sql(&sql).execute(&pool).await.map_err(|e| BizError::msg(e.to_string()))?;
        return Ok(json!(rs.rows_affected()));
    }
    let sql = format!("select * from ({}) limit {}", sql, MAX_SELECT_LIMIT);
    let rows = sqlx::query(&sql).fetch_all(&pool).await.map_err(|e| BizError::msg(e.to_string()))?;
    let columns: Vec<String> = match rows.first() {
        Some(row) => row.columns().iter().map(|it| it.name().to_string()).collect(),
        None => pool
            .describe(&sql)
            .await
            .map(|it| it.columns().iter().map(|c| c.name().to_string()).collect())
            .unwrap_or_default(),
    };
    let data: Vec<Value> = rows.iter().map(|it| Value::Object(table_data::row_to_json(it))).collect();
    Ok(json!({ "columns": columns, "data": data }))
}

/// 开启文件处理
#[POST("/api/advanced/re_handle")]
pub async fn re_handle() {
    file_handle_service::notify();
}

/// 获取存储文件占用的空间
#[GET("/api/advanced/used_size")]
pub async fn used_size() -> Result<String, BizError> {
    let mut total = 0u64;
    let mut missing = 0;
    for it in storage_file_dao::select_all(&mut lib_db::get_context()).await? {
        match tokio::fs::metadata(&it.path).await {
            Ok(meta) => total += meta.len(),
            Err(_) => missing += 1,
        }
    }
    let mut text = format!("{}({}B)", total.data_size(), total);
    if missing > 0 {
        text += &format!(",丢失{}个文件", missing);
    }
    Ok(text)
}

/// 立即回收没有被使用的存储文件(在后台执行)
#[POST("/api/advanced/recycle_now")]
pub async fn recycle_now() {
    tokio::spawn(async {
        let time = time_util::current_millis() - RECYCLE_NOW_SAFE_MILLIS;
        if let Err(e) = recycle_service::recycle_not_use_storage(time).await {
            eprintln!("-->回收未使用的文件失败: {}", e);
        }
    });
}

/// 立即执行一次回收定时任务(在后台执行)
#[POST("/api/advanced/recycle_timer")]
pub async fn recycle_timer() {
    tokio::spawn(recycle_service::run());
}

/// 备份数据库
#[POST("/api/advanced/db_backup")]
pub async fn db_backup() -> Result<String, BizError> {
    let path = db_backup_util::backup().await.map_err(BizError::msg)?;
    Ok(path.to_string_lossy().to_string())
}

/// 重置并同步本机数据(只允许在只读模式下执行)
#[POST("/api/advanced/reset_data")]
pub async fn reset_data() -> Result<(), BizError> {
    if !system_config::get().is_read_only {
        return Err(biz_error!("该操作只允许本机在只读模式下执行"));
    }

    //直接操作数据库,不记录同步日志
    let sql = "delete from dfs_file;delete from dfs_file_delete;delete from share;delete from sql_log;delete from user;delete from user_token;";
    sqlx::raw_sql(sql).execute(&lib_db::get()).await?;
    tokio::spawn(sync::sync_by_table::sync_all());
    Ok(())
}

/// 开始验证文件完整,正在验证时则停止验证
#[POST("/api/advanced/validate_file")]
pub async fn validate_file() {
    {
        let mut state = VALIDATE.lock().unwrap();
        if state.is_running {
            state.is_stop = true;
            return;
        }
        *state = model::ValidateState {
            is_running: true,
            ..Default::default()
        };
    }
    tokio::spawn(async {
        let list = storage_file_dao::select_all(&mut lib_db::get_context()).await.unwrap_or_default();
        VALIDATE.lock().unwrap().total = list.len();
        for (i, it) in list.iter().enumerate() {
            if VALIDATE.lock().unwrap().is_stop {
                break;
            }
            let error = match file_util::md5(std::path::Path::new(&it.path)).await {
                Err(_) => Some(format!("{}：找不到文件路径：{}", it.md5, it.path)),
                Ok(md5) if md5 != it.md5 => Some(format!("{}：文件不完整，当前文件MD5：{}", it.md5, md5)),
                Ok(_) => None,
            };
            let mut state = VALIDATE.lock().unwrap();
            if let Some(error) = error {
                state.errors.push(error);
            }
            state.current = i + 1;
        }
        let mut state = VALIDATE.lock().unwrap();
        if !state.is_stop && state.errors.is_empty() {
            state.errors.push(format!("文件数据完整，检测时间：{}", time_util::current_millis().date_format()));
        }
        state.is_running = false;
    });
}

/// 获取验证文件完整的进度
#[GET("/api/advanced/validate_file")]
pub async fn validate_file_state() -> model::ValidateState {
    VALIDATE.lock().unwrap().clone()
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AdvancedInfo {

        // 文件处理状态
        pub file_handling: String,

        // 回收定时器状态
        pub recycle_storage_timer_state: String,

        // 磁盘使用状态
        pub storage_state: String,
    }

    #[derive(Debug, Default, Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ValidateState {

        // 是否正在验证
        pub is_running: bool,

        // 是否被停止
        #[serde(skip)]
        pub is_stop: bool,

        // 文件总数
        pub total: usize,

        // 已经验证的数量
        pub current: usize,

        // 验证失败的文件
        pub errors: Vec<String>,
    }
}
