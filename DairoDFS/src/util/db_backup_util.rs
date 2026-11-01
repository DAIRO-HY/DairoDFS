use crate::{application, system_config};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

/// 数据库备份目录(与老版本一致)
static BACKUP_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&application::ARGS.data_dir).join("db_backup"));

/// 数据库备份文件名格式(与老版本一致)
const FILE_NAME_FORMAT: &str = "%Y%m%d%H%M%S";

/// 备份数据库,并删除过期的备份文件
/// 先用VACUUM INTO生成一份一致的数据库快照再压缩,备份期间不需要关闭数据库连接
pub async fn backup() -> Result<PathBuf, String> {
    tokio::fs::create_dir_all(BACKUP_PATH.as_path()).await.map_err(|e| e.to_string())?;
    let name = Local::now().format(FILE_NAME_FORMAT).to_string();
    let zip_path = BACKUP_PATH.join(format!("{}.zip", name));
    if zip_path.exists() {
        return Err("1秒内不能重复备份".to_string());
    }
    let snapshot = application::TEMP_PATH.join(format!("backup-{}.sqlite", name));
    let _ = tokio::fs::remove_file(&snapshot).await;
    let sql = format!("VACUUM INTO '{}'", snapshot.to_string_lossy().replace('\'', "''"));
    sqlx::query(&sql).execute(&lib_db::get()).await.map_err(|e| format!("备份数据库失败:{}", e))?;
    let zip_path_clone = zip_path.clone();
    let snapshot_clone = snapshot.clone();
    let result = tokio::task::spawn_blocking(move || write_zip(&snapshot_clone, &zip_path_clone))
        .await
        .map_err(|e| e.to_string())?;
    let _ = tokio::fs::remove_file(&snapshot).await;
    if let Err(e) = result {
        let _ = tokio::fs::remove_file(&zip_path).await;
        return Err(format!("压缩备份文件失败:{}", e));
    }
    delete_expire_file().await;
    Ok(zip_path)
}

/// 把数据库快照压缩成zip(最高压缩比),zip里的文件名与数据库文件名一致
fn write_zip(snapshot: &PathBuf, zip_path: &PathBuf) -> Result<(), String> {
    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let now = Local::now();
    let mut options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .large_file(true);
    if let Ok(time) = zip::DateTime::from_date_and_time(
        now.year() as u16,
        now.month() as u8,
        now.day() as u8,
        now.hour() as u8,
        now.minute() as u8,
        now.second() as u8,
    ) {
        options = options.last_modified_time(time);
    }
    zip.start_file(lib_db::DB_FILE_NAME, options).map_err(|e| e.to_string())?;
    let mut source = std::fs::File::open(snapshot).map_err(|e| e.to_string())?;
    std::io::copy(&mut source, &mut zip).map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?.flush().map_err(|e| e.to_string())
}

/// 删除过期的备份文件
async fn delete_expire_file() {
    let expire = system_config::get().db_backup_expire_day * 24 * 60 * 60 * 1000;
    let now = Local::now().timestamp_millis();
    let Ok(mut entries) = tokio::fs::read_dir(BACKUP_PATH.as_path()).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(time) = name.strip_suffix(".zip") else {
            continue;
        };
        let Ok(time) = NaiveDateTime::parse_from_str(time, FILE_NAME_FORMAT) else {
            continue;
        };
        let Some(time) = Local.from_local_datetime(&time).earliest() else {
            continue;
        };
        if now - time.timestamp_millis() > expire {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}
