use rust_embed::RustEmbed;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Connection, SqliteConnection, SqlitePool};
use sqlx_context::DbContext;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

#[derive(RustEmbed)]
#[folder = "../DairoDFS/assets/ddl/"]
struct Assets;

// VERSION 数据库版本号
const VERSION: i32 = 1;

/// 数据库文件名
pub const DB_FILE_NAME: &str = "dairo-dfs2.sqlite";

/// 老版本(Go版)的数据库文件名,第一次启动时从这个库迁移数据
const OLD_DB_FILE_NAME: &str = "dairo-dfs.sqlite";

/// 建表顺序
const SQL_FILES: [&str; 7] = [
    "dfs_file.sql",
    "dfs_file_delete.sql",
    "storage_file.sql",
    "share.sql",
    "sql_log.sql",
    "user.sql",
    "user_token.sql",
];

/// 需要从老库迁移数据的表(sql_log是Go版的同步日志,新版不再使用,不迁移)
const MIGRATE_TABLES: [&str; 6] = [
    "dfs_file",
    "dfs_file_delete",
    "storage_file",
    "share",
    "user",
    "user_token",
];

/// 获取数据目录下的数据库文件路径
pub fn db_path(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join(DB_FILE_NAME)
}

/// 全局数据库连接池
static DB_CONN: OnceLock<SqlitePool> = OnceLock::new();

/// 获取连接池
pub fn get() -> SqlitePool {
    DB_CONN.get().unwrap().clone()
}

/// 获取一个数据库操作上下文
pub fn get_context() -> DbContext {
    DbContext::new(DB_CONN.get().unwrap().clone())
}

/// 连接参数
fn connect_options(db_path: &Path) -> SqliteConnectOptions {
    //直接传文件路径,不拼接成URL,路径里有?、#等字符时也能正常打开
    SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        //WAL模式:多个读者,较少写者的场景(如Web服务器)
        .journal_mode(SqliteJournalMode::Wal)
        //数据库被锁时的等待时间
        .busy_timeout(Duration::from_secs(10))
}

// 初始化数据库连接和表结构
// - data_dir 数据目录,数据库文件保存在该目录下
pub async fn init(data_dir: &str) {
    //SQLite不会自动创建目录,先确保数据目录存在
    std::fs::create_dir_all(data_dir).unwrap();
    let db_path = db_path(data_dir);
    eprintln!("-->数据库初始化中...");
    init_db(data_dir, &db_path).await.unwrap_or_else(|it| {
        eprintln!("-->数据库初始化失败: {}", it);
        std::process::exit(1);
    });
    eprintln!("-->数据库初始化完成。");
}

/// 编译期初始化数据库(仅建表),供build.rs生成DAO和sqlx宏检查sql语句使用
pub async fn init_build(db_path: &Path) {
    //每次都重新建库,保证表结构和ddl一致
    let _ = std::fs::remove_file(db_path);
    init_pool(db_path);
    upgrade().await.unwrap();
}

async fn init_db(data_dir: &str, db_path: &Path) -> Result<(), Box<dyn Error>> {
    let old_db_path = Path::new(data_dir).join(OLD_DB_FILE_NAME);
    if !db_path.exists() && old_db_path.exists() {
        //新库还不存在,而老库存在,说明是从老版本升级上来的,先迁移数据
        migrate_from_old(&old_db_path, db_path).await?;
    }
    init_pool(db_path);

    //升级数据库（如果需要）
    upgrade().await?;
    Ok(())
}

/// 初始化全局连接池
fn init_pool(db_path: &Path) {
    DB_CONN
        .set(
            SqlitePoolOptions::new()
                .max_connections(20)
                .connect_lazy_with(connect_options(db_path)),
        )
        .unwrap();
}

// 数据库升级
async fn upgrade() -> Result<(), Box<dyn Error>> {
    let mut conn = get().acquire().await?;
    let version: i64 = sqlx::query_scalar("PRAGMA USER_VERSION")
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(0);
    if version == 0 {
        //创建表
        create_table(&mut *conn).await?;
    }

    //设置数据库版本号(版本号没变时不写库)
    if version != VERSION as i64 {
        sqlx::query(&(format!("PRAGMA USER_VERSION = {}", VERSION)))
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

// 创建表
async fn create_table(conn: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
    for sql_file in SQL_FILES {
        let Some(sql_mbed) = Assets::get(sql_file) else {
            return Err(From::from(format!("{} not found", sql_file)));
        };
        let sql_str = std::str::from_utf8(sql_mbed.data.as_ref())?;
        sqlx::raw_sql(sql_str).execute(&mut *conn).await?;
    }
    Ok(())
}

/// 从老版本数据库迁移数据
/// 先在临时文件里建好新库并复制数据,校验通过后再改名为正式文件,中途失败或断电时下次启动会重新迁移。老库只读,不做任何修改
async fn migrate_from_old(old_db_path: &Path, db_path: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("-->检测到老版本数据库:{},开始迁移数据...", old_db_path.display());
    let temp_path = db_path.with_extension("sqlite.tmp");
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{}", temp_path.display(), suffix));
    }

    //迁移时不需要WAL,用DELETE模式保证提交后数据都在主文件里,改名时不会遗漏
    let mut conn = SqliteConnection::connect_with(
        &connect_options(&temp_path).journal_mode(SqliteJournalMode::Delete),
    )
    .await?;
    create_table(&mut conn).await?;
    //以只读方式挂载老库:读写方式挂载时,关闭连接会把老库的WAL日志合并进主文件,导致老库被修改
    sqlx::query("ATTACH DATABASE ? AS old")
        .bind(format!("file:{}?mode=ro", uri_path(old_db_path)))
        .execute(&mut conn)
        .await?;

    let mut tx = conn.begin().await?;
    for table in MIGRATE_TABLES {
        let new_columns = table_columns(&mut tx, "main", table).await?;
        let old_columns = table_columns(&mut tx, "old", table).await?;

        //按列名对应复制,user_token.version改名为client_version
        let select_columns = new_columns
            .iter()
            .map(|it| {
                let old_name = if table == "user_token" && it == "client_version" {
                    "version"
                } else {
                    it.as_str()
                };
                if old_columns.iter().any(|old| old == old_name) {
                    format!("\"{}\"", old_name)
                } else {
                    //老库没有这一列,用新表的默认值
                    "NULL".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let insert_columns = new_columns
            .iter()
            .map(|it| format!("\"{}\"", it))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "insert into main.{table}({insert_columns}) select {select_columns} from old.{table}"
        );
        sqlx::query(&sql).execute(&mut *tx).await?;

        //校验数据条数
        let old_count: i64 = sqlx::query_scalar(&format!("select count(*) from old.{table}"))
            .fetch_one(&mut *tx)
            .await?;
        let new_count: i64 = sqlx::query_scalar(&format!("select count(*) from main.{table}"))
            .fetch_one(&mut *tx)
            .await?;
        if old_count != new_count {
            return Err(From::from(format!(
                "表{}迁移后数据条数不一致,老库:{},新库:{}",
                table, old_count, new_count
            )));
        }
        eprintln!("-->表{}迁移完成,共{}条", table, new_count);
    }
    tx.commit().await?;

    //迁移完成,设置数据库版本号
    sqlx::query(&(format!("PRAGMA USER_VERSION = {}", VERSION)))
        .execute(&mut conn)
        .await?;
    sqlx::query("DETACH DATABASE old").execute(&mut conn).await?;
    conn.close().await?;
    std::fs::rename(&temp_path, db_path)?;
    eprintln!("-->数据迁移完成:{}", db_path.display());
    Ok(())
}

/// 获取表的列名(按定义顺序)
async fn table_columns(
    conn: &mut SqliteConnection,
    schema: &str,
    table: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(&format!("select name from pragma_table_info('{table}', '{schema}')"))
        .fetch_all(conn)
        .await
}

/// 将文件路径转换成SQLite URI中的路径部分(转义URI中有特殊含义的字符,Windows的反斜杠换成斜杠)
fn uri_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "/")
        .replace('%', "%25")
        .replace('?', "%3F")
        .replace('#', "%23")
}
