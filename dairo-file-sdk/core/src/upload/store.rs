use crate::error::SdkResult;
use crate::index::Index;
use rusqlite::{params, OptionalExtension, Row};

/// 数据来源:本地文件路径
pub const SOURCE_FILE: i64 = 0;

/// 数据来源:由平台提供数据
pub const SOURCE_CUSTOM: i64 = 1;

/// 一条上传记录
#[derive(Debug, Clone)]
pub struct UploadEntry {
    pub id: i64,

    /// 上传到哪个账号
    pub account: String,

    /// 数据来源类型
    pub source_type: i64,

    /// 本地文件路径,或者平台定义的数据来源ID
    pub source: String,

    /// 上传到DFS的文件路径(包含文件名)
    pub dfs_path: String,

    /// 文件大小,还没有读取时为None
    pub size: Option<u64>,

    /// 已经上传的大小
    pub uploaded: u64,

    /// 文件MD5,还没有计算时为None
    pub md5: Option<String>,

    /// 文件类型,不指定时DFS按扩展名判断
    pub content_type: Option<String>,

    /// 缩略图地址,用于上传列表显示
    pub thumb: Option<String>,

    /// 自己的文件里已经有相同的文件时跳过
    pub skip_if_exists: bool,

    /// 状态
    pub state: i64,

    /// 上传失败的错误信息
    pub error: Option<String>,

    /// 创建时间(毫秒)
    pub create_date: i64,

    /// 上传完成时间(毫秒)
    pub finish_date: Option<i64>,
}

impl UploadEntry {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            account: row.get("account")?,
            source_type: row.get("source_type")?,
            source: row.get("source")?,
            dfs_path: row.get("dfs_path")?,
            size: row.get::<_, Option<i64>>("size")?.map(|it| it as u64),
            uploaded: row.get::<_, i64>("uploaded")? as u64,
            md5: row.get("md5")?,
            content_type: row.get("content_type")?,
            thumb: row.get("thumb")?,
            skip_if_exists: row.get::<_, i64>("skip_if_exists")? != 0,
            state: row.get("state")?,
            error: row.get("error")?,
            create_date: row.get("create_date")?,
            finish_date: row.get("finish_date")?,
        })
    }
}

impl Index {
    pub fn add_upload(&self, it: &UploadEntry) -> SdkResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "insert into upload (account, source_type, source, dfs_path, size, md5, content_type, thumb, skip_if_exists, state, create_date)
             values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                it.account,
                it.source_type,
                it.source,
                it.dfs_path,
                it.size.map(|it| it as i64),
                it.md5,
                it.content_type,
                it.thumb,
                it.skip_if_exists as i64,
                it.state,
                it.create_date
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_upload(&self, id: i64) -> SdkResult<Option<UploadEntry>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("select * from upload where id = ?", [id], UploadEntry::from_row).optional()?)
    }

    /// 按条件分页查询,按创建顺序
    pub fn select_uploads(&self, account: Option<&str>, state: Option<i64>, offset: u64, limit: u64) -> SdkResult<Vec<UploadEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "select * from upload where (? is null or account = ?) and (? is null or state = ?) order by id limit ? offset ?",
        )?;
        let list = stmt
            .query_map(params![account, account, state, state, limit as i64, offset as i64], UploadEntry::from_row)?
            .collect::<Result<_, _>>()?;
        Ok(list)
    }

    /// 查询下一个等待上传的任务
    pub fn next_upload(&self, account: &str, state: i64, exclude: &[i64]) -> SdkResult<Option<UploadEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("select * from upload where account = ? and state = ? order by id")?;
        let mut rows = stmt.query(params![account, state])?;
        while let Some(row) = rows.next()? {
            let it = UploadEntry::from_row(row)?;
            if !exclude.contains(&it.id) {
                return Ok(Some(it));
            }
        }
        Ok(None)
    }

    pub fn set_upload_state(&self, id: i64, state: i64, error: Option<&str>) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("update upload set state = ?, error = ? where id = ?", params![state, error, id])?;
        Ok(())
    }

    /// 把一个账号下指定状态的任务改成另一个状态
    pub fn change_upload_state(&self, account: Option<&str>, from: i64, to: i64) -> SdkResult<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("select id from upload where (? is null or account = ?) and state = ?")?;
        let ids: Vec<i64> = stmt.query_map(params![account, account, from], |row| row.get(0))?.collect::<Result<_, _>>()?;
        conn.execute(
            "update upload set state = ?, error = null where (? is null or account = ?) and state = ?",
            params![to, account, account, from],
        )?;
        Ok(ids)
    }

    pub fn set_upload_md5(&self, id: i64, size: u64, md5: &str) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("update upload set size = ?, md5 = ? where id = ?", params![size as i64, md5, id])?;
        Ok(())
    }

    pub fn set_upload_progress(&self, id: i64, uploaded: u64) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("update upload set uploaded = ? where id = ?", params![uploaded as i64, id])?;
        Ok(())
    }

    pub fn finish_upload(&self, id: i64, state: i64, time: i64) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "update upload set state = ?, error = null, uploaded = coalesce(size, uploaded), finish_date = ? where id = ?",
            params![state, time, id],
        )?;
        Ok(())
    }

    pub fn delete_upload(&self, id: i64) -> SdkResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("delete from upload where id = ?", [id])?;
        Ok(())
    }
}
