use crate::error::SdkResult;
use crate::options::CacheMode;
use crate::range_set::RangeSet;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::Path;
use std::sync::Mutex;

/// 状态:等待下载
pub const STATE_WAITING: i64 = 0;

/// 状态:下载中
pub const STATE_DOWNLOADING: i64 = 1;

/// 状态:暂停中
pub const STATE_PAUSED: i64 = 2;

/// 状态:下载失败
pub const STATE_ERROR: i64 = 3;

/// 状态:下载完成
pub const STATE_FINISH: i64 = 10;

/// 状态:上传时已经存在,跳过
pub const STATE_SKIPPED: i64 = 11;

const CREATE_SQL: &str = "
create table if not exists cache (
    id text primary key not null,
    namespace text not null,
    key text not null,
    url text not null,
    grp text not null,
    mode integer not null,
    name text,
    ext text not null,
    thumb text,
    size integer,
    downloaded integer not null default 0,
    ranges text not null default '',
    etag text,
    last_modified text,
    content_type text,
    md5 text,
    headers text not null default '[]',
    state integer not null,
    error text,
    create_date integer not null,
    finish_date integer,
    access_date integer not null
);
create index if not exists idx_cache_grp on cache (grp);
create index if not exists idx_cache_access_date on cache (access_date);
create index if not exists idx_cache_mode_state on cache (mode, state);
create index if not exists idx_cache_md5 on cache (md5);

create table if not exists upload (
    id integer primary key autoincrement,
    account text not null,
    source_type integer not null,
    source text not null,
    dfs_path text not null,
    size integer,
    uploaded integer not null default 0,
    md5 text,
    content_type text,
    thumb text,
    skip_if_exists integer not null default 0,
    state integer not null,
    error text,
    create_date integer not null,
    finish_date integer
);
create index if not exists idx_upload_account_state on upload (account, state);
";

/// 一条缓存记录
#[derive(Debug, Clone)]
pub struct Entry {
    /// 缓存ID
    pub id: String,

    /// 命名空间
    pub namespace: String,

    /// 命名空间内的key
    pub key: String,

    /// 最后一次使用的下载地址
    pub url: String,

    /// 缓存分组
    pub group: String,

    /// 缓存方式
    pub mode: CacheMode,

    /// 文件名
    pub name: Option<String>,

    /// 缓存文件的扩展名
    pub ext: String,

    /// 缩略图地址
    pub thumb: Option<String>,

    /// 文件大小,未知时为None
    pub size: Option<u64>,

    /// 已经下载的区间
    pub ranges: RangeSet,

    /// 服务器返回的ETag,断点续传时用来确认文件没有改变
    pub etag: Option<String>,

    /// 服务器返回的Last-Modified,没有ETag时使用
    pub last_modified: Option<String>,

    /// 服务器返回的文件类型
    pub content_type: Option<String>,

    /// 期望的文件MD5
    pub md5: Option<String>,

    /// 下载时附加的请求头
    pub headers: Vec<(String, String)>,

    /// 状态
    pub state: i64,

    /// 下载失败的错误信息
    pub error: Option<String>,

    /// 创建时间(毫秒)
    pub create_date: i64,

    /// 下载完成时间(毫秒)
    pub finish_date: Option<i64>,

    /// 最后访问时间(毫秒)
    pub access_date: i64,
}

impl Entry {
    /// 缓存文件相对于数据目录的路径
    pub fn file(&self) -> String {
        let name = if self.ext.is_empty() { self.id.clone() } else { format!("{}.{}", self.id, self.ext) };
        format!("files/{}/{}", &self.id[..2], name)
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let headers: String = row.get("headers")?;
        Ok(Self {
            id: row.get("id")?,
            namespace: row.get("namespace")?,
            key: row.get("key")?,
            url: row.get("url")?,
            group: row.get("grp")?,
            mode: CacheMode::from_i64(row.get("mode")?),
            name: row.get("name")?,
            ext: row.get("ext")?,
            thumb: row.get("thumb")?,
            size: row.get::<_, Option<i64>>("size")?.map(|it| it as u64),
            ranges: RangeSet::parse(&row.get::<_, String>("ranges")?),
            etag: row.get("etag")?,
            last_modified: row.get("last_modified")?,
            content_type: row.get("content_type")?,
            md5: row.get("md5")?,
            headers: serde_json::from_str(&headers).unwrap_or_default(),
            state: row.get("state")?,
            error: row.get("error")?,
            create_date: row.get("create_date")?,
            finish_date: row.get("finish_date")?,
            access_date: row.get("access_date")?,
        })
    }
}

/// 缓存索引数据库
pub struct Index {
    pub(crate) conn: Mutex<Connection>,
}

impl Index {
    pub fn open(path: &Path) -> SdkResult<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(CREATE_SQL)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn get(&self, id: &str) -> SdkResult<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("select * from cache where id = ?", [id], Entry::from_row).optional()?)
    }

    /// 添加或者覆盖一条记录
    pub fn save(&self, it: &Entry) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "insert or replace into cache (id, namespace, key, url, grp, mode, name, ext, thumb, size, downloaded, ranges, etag,
             last_modified, content_type, md5, headers, state, error, create_date, finish_date, access_date)
             values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                it.id,
                it.namespace,
                it.key,
                it.url,
                it.group,
                it.mode as i64,
                it.name,
                it.ext,
                it.thumb,
                it.size.map(|it| it as i64),
                it.ranges.len() as i64,
                it.ranges.to_string(),
                it.etag,
                it.last_modified,
                it.content_type,
                it.md5,
                serde_json::to_string(&it.headers).unwrap(),
                it.state,
                it.error,
                it.create_date,
                it.finish_date,
                it.access_date,
            ],
        )?;
        Ok(())
    }

    /// 保存下载进度
    pub fn save_progress(&self, it: &Entry) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "update cache set size = ?, downloaded = ?, ranges = ?, etag = ?, last_modified = ?, content_type = ? where id = ?",
            params![
                it.size.map(|it| it as i64),
                it.ranges.len() as i64,
                it.ranges.to_string(),
                it.etag,
                it.last_modified,
                it.content_type,
                it.id
            ],
        )?;
        Ok(())
    }

    pub fn set_state(&self, id: &str, state: i64, error: Option<&str>) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("update cache set state = ?, error = ? where id = ?", params![state, error, id])?;
        Ok(())
    }

    /// 批量更新最后访问时间
    pub fn touch(&self, ids: &[String], time: i64) -> SdkResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare("update cache set access_date = ? where id = ?")?;
            for id in ids {
                stmt.execute(params![time, id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("delete from cache where id = ?", [id])?;
        Ok(())
    }

    /// 按条件分页查询,按创建时间倒序
    pub fn select(&self, group: Option<&str>, mode: Option<CacheMode>, state: Option<i64>, offset: u64, limit: u64) -> SdkResult<Vec<Entry>> {
        let mut sql = "select * from cache where 1 = 1".to_string();
        let mut args: Vec<rusqlite::types::Value> = Vec::new();
        if let Some(group) = group {
            sql += " and grp = ?";
            args.push(group.to_string().into());
        }
        if let Some(mode) = mode {
            sql += " and mode = ?";
            args.push((mode as i64).into());
        }
        if let Some(state) = state {
            sql += " and state = ?";
            args.push(state.into());
        }
        sql += " order by create_date desc limit ? offset ?";
        args.push((limit as i64).into());
        args.push((offset as i64).into());
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        let list = stmt.query_map(rusqlite::params_from_iter(args), Entry::from_row)?.collect::<Result<_, _>>()?;
        Ok(list)
    }

    /// 按分组统计数量和已经下载的大小
    pub fn stats(&self, mode: Option<CacheMode>) -> SdkResult<Vec<(String, u64, u64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "select grp, count(*), coalesce(sum(downloaded), 0) from cache where ? is null or mode = ? group by grp order by grp",
        )?;
        let mode = mode.map(|it| it as i64);
        let list = stmt
            .query_map(params![mode, mode], |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64, row.get::<_, i64>(2)? as u64)))?
            .collect::<Result<_, _>>()?;
        Ok(list)
    }

    /// 查询可以被自动清理的缓存(不缓存和缓存),按最后访问时间升序
    /// - before 只查询最后访问时间在这之前的
    pub fn select_cleanable(&self, before: i64) -> SdkResult<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("select * from cache where mode < ? and access_date < ? order by access_date")?;
        let list = stmt.query_map(params![CacheMode::Save as i64, before], Entry::from_row)?.collect::<Result<_, _>>()?;
        Ok(list)
    }

    /// 可以被自动清理的缓存的总大小
    pub fn cleanable_size(&self) -> SdkResult<u64> {
        let conn = self.conn.lock().unwrap();
        let size: i64 = conn.query_row(
            "select coalesce(sum(downloaded), 0) from cache where mode < ?",
            [CacheMode::Save as i64],
            |row| row.get(0),
        )?;
        Ok(size as u64)
    }

    /// 查询一个已经下载完成的相同MD5的文件
    pub fn select_finished_by_md5(&self, md5: &str, exclude_id: &str) -> SdkResult<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "select * from cache where md5 = ? collate nocase and state = ? and id != ? limit 1",
                params![md5, STATE_FINISH, exclude_id],
                Entry::from_row,
            )
            .optional()?)
    }

    /// 查询指定缓存方式的记录
    pub fn select_by_mode(&self, mode: CacheMode) -> SdkResult<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("select * from cache where mode = ? order by create_date")?;
        let list = stmt.query_map([mode as i64], Entry::from_row)?.collect::<Result<_, _>>()?;
        Ok(list)
    }
}
