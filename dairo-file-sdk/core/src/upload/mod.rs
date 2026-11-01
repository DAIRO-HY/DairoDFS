//! 上传文件到DFS
//!
//! 流程:计算MD5 → (可选)检查自己的文件里是否已经存在 → 秒传(by_md5) → 查询已经上传的大小 → 断点续传(by_stream) → 秒传
//! 上传任务保存在数据库里,App重启之后继续上传;任务属于某个账号,只有当前账号的任务会上传

mod store;
mod worker;

use crate::error::{SdkError, SdkResult};
use crate::event::{Event, UploadState};
use crate::index::{STATE_DOWNLOADING, STATE_ERROR, STATE_FINISH, STATE_PAUSED, STATE_SKIPPED, STATE_WAITING};
use crate::{now_millis, FileSdk, Inner};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use store::{UploadEntry, SOURCE_CUSTOM, SOURCE_FILE};
use tokio::sync::watch;

/// 上传的数据来源
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadSource {
    /// 本地文件路径
    File(String),

    /// 由平台提供数据,值是平台自己定义的ID(例如iOS相册的localIdentifier),通过UploadSourceProvider读取
    Custom(String),
}

/// 平台提供上传数据,用于无法直接通过文件路径读取的数据(iOS相册、实况照片、需要授权才能访问的文件等)
/// 在SDK的后台线程里调用,可以在这里做耗时的操作
pub trait UploadSourceProvider: Send + Sync {
    /// 数据的总大小
    fn size(&self, source: String) -> Result<u64, String>;

    /// 读取数据,返回的数据长度必须等于len(最后一段除外)
    fn read(&self, source: String, offset: u64, len: u64) -> Result<Vec<u8>, String>;

    /// 读取结束,可以在这里释放资源
    fn close(&self, source: String) {
        let _ = source;
    }
}

/// 要上传的文件
#[derive(Debug, Clone)]
pub struct UploadItem {
    pub source: UploadSource,

    /// 上传到DFS的文件路径(包含文件名),例如:/相册/1.jpg
    pub dfs_path: String,

    /// 文件MD5,平台已经计算过时传入,可以省去计算的时间
    pub md5: Option<String>,

    /// 文件类型,不指定时DFS按扩展名判断
    pub content_type: Option<String>,

    /// 缩略图地址,用于上传列表显示
    pub thumb: Option<String>,

    /// 自己的文件里已经有相同的文件时跳过(相册备份时使用)
    pub skip_if_exists: bool,
}

impl UploadItem {
    pub fn new(source: UploadSource, dfs_path: impl Into<String>) -> Self {
        Self { source, dfs_path: dfs_path.into(), md5: None, content_type: None, thumb: None, skip_if_exists: false }
    }
}

/// 上传信息
#[derive(Debug, Clone, PartialEq)]
pub struct UploadInfo {
    pub id: i64,
    pub account: String,
    pub source: UploadSource,
    pub dfs_path: String,

    /// 文件名
    pub name: String,

    /// 文件大小,还没有读取时为None
    pub size: Option<u64>,

    /// 已经上传的大小
    pub uploaded: u64,
    pub md5: Option<String>,
    pub thumb: Option<String>,
    pub state: UploadState,

    /// 上传失败的错误信息
    pub error: Option<String>,

    /// 创建时间(毫秒)
    pub create_date: i64,

    /// 上传完成时间(毫秒)
    pub finish_date: Option<i64>,
}

/// 上传列表查询条件
#[derive(Debug, Clone)]
pub struct UploadQuery {
    /// 账号,None时查询所有账号
    pub account: Option<String>,
    pub state: Option<UploadState>,
    pub offset: u64,
    pub limit: u64,
}

impl Default for UploadQuery {
    fn default() -> Self {
        Self { account: None, state: None, offset: 0, limit: 100 }
    }
}

/// 当前登录的DFS账号
#[derive(Debug, Clone)]
pub(crate) struct Account {
    pub name: String,

    /// DFS服务器地址,例如:http://192.168.1.2:8031
    pub server: String,
    pub token: String,
}

/// 正在上传的任务
pub(crate) struct Running {
    stop: watch::Sender<bool>,

    /// 被停止之后的状态(等待或暂停)
    stop_state: AtomicI64,
}

impl Running {
    pub fn is_stopped(&self) -> bool {
        *self.stop.borrow()
    }

    pub async fn stopped(&self) {
        let mut rx = self.stop.subscribe();
        let _ = rx.wait_for(|it| *it).await;
    }
}

#[derive(Default)]
pub(crate) struct Uploads {
    account: Mutex<Option<Account>>,
    provider: Mutex<Option<Arc<dyn UploadSourceProvider>>>,

    /// 上传ID => 正在上传的任务
    running: Mutex<HashMap<i64, Arc<Running>>>,
}

impl Uploads {
    pub fn account(&self) -> Option<Account> {
        self.account.lock().unwrap().clone()
    }

    pub fn provider(&self) -> Option<Arc<dyn UploadSourceProvider>> {
        self.provider.lock().unwrap().clone()
    }

    /// 在并发数允许的范围内开始上传当前账号的任务
    pub fn schedule(&self, inner: &Arc<Inner>) {
        let Some(account) = self.account() else {
            return;
        };
        let max = inner.config().max_uploading_count;
        let mut running = self.running.lock().unwrap();
        while running.len() < max {
            let exclude: Vec<i64> = running.keys().copied().collect();
            let Ok(Some(entry)) = inner.index.next_upload(&account.name, STATE_WAITING, &exclude) else {
                break;
            };
            let _ = inner.index.set_upload_state(entry.id, STATE_DOWNLOADING, None);
            inner.events.upload_state(entry.id, STATE_DOWNLOADING, None);
            let task = Arc::new(Running { stop: watch::Sender::new(false), stop_state: AtomicI64::new(STATE_WAITING) });
            running.insert(entry.id, task.clone());
            let inner = inner.clone();
            inner.rt.clone().spawn(async move {
                let id = entry.id;
                let result = worker::upload(&inner, entry, &task).await;
                inner.uploads.running.lock().unwrap().remove(&id);
                match result {
                    Ok(state) => {
                        let _ = inner.index.finish_upload(id, state, now_millis());
                        inner.events.upload_state(id, state, None);
                    }
                    Err(SdkError::Cancelled) => {
                        let state = task.stop_state.load(Ordering::Relaxed);
                        let _ = inner.index.set_upload_state(id, state, None);
                        inner.events.upload_state(id, state, None);
                    }
                    Err(e) => {
                        let error = e.to_string();
                        let _ = inner.index.set_upload_state(id, STATE_ERROR, Some(&error));
                        inner.events.upload_state(id, STATE_ERROR, Some(&error));
                    }
                }
                inner.uploads.schedule(&inner);
            });
        }
    }

    /// 停止正在上传的任务
    /// - state 停止之后的状态
    /// - return 是否正在上传
    fn stop(&self, id: i64, state: i64) -> bool {
        let Some(task) = self.running.lock().unwrap().get(&id).cloned() else {
            return false;
        };
        task.stop_state.store(state, Ordering::Relaxed);
        task.stop.send_replace(true);
        true
    }

    /// 停止所有正在上传的任务,改为等待上传
    fn stop_all(&self) {
        let ids: Vec<i64> = self.running.lock().unwrap().keys().copied().collect();
        for id in ids {
            self.stop(id, STATE_WAITING);
        }
    }

    /// 等待任务停止
    async fn wait_stopped(&self, id: i64) {
        while self.running.lock().unwrap().contains_key(&id) {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

impl FileSdk {
    /// 设置当前登录的DFS账号,只有这个账号的任务会上传
    /// 切换账号时,正在上传的其他账号的任务会停止;同一个账号再次调用时只更新服务器地址和票据(例如票据过期之后重新登录)
    /// - account 账号标识,由平台定义,建议使用"集群ID:用户ID"
    /// - server DFS服务器地址,例如:http://192.168.1.2:8031
    /// - token 登录票据
    pub fn set_dfs_account(&self, account: &str, server: &str, token: &str) {
        let uploads = &self.inner.uploads;
        let changed = uploads.account().is_none_or(|it| it.name != account);
        if changed {
            uploads.stop_all();
        }
        *uploads.account.lock().unwrap() = Some(Account {
            name: account.to_string(),
            server: server.trim_end_matches('/').to_string(),
            token: token.to_string(),
        });
        uploads.schedule(&self.inner);
    }

    /// 退出登录,停止所有上传,任务保留,下次设置同一个账号时继续上传
    pub fn clear_dfs_account(&self) {
        let uploads = &self.inner.uploads;
        *uploads.account.lock().unwrap() = None;
        uploads.stop_all();
    }

    /// 设置平台提供上传数据的回调
    pub fn set_upload_source_provider(&self, provider: Option<Arc<dyn UploadSourceProvider>>) {
        *self.inner.uploads.provider.lock().unwrap() = provider;
    }

    /// 添加上传任务到当前账号,返回上传ID
    pub fn add_uploads(&self, items: &[UploadItem]) -> SdkResult<Vec<i64>> {
        let account = self.inner.uploads.account().ok_or(SdkError::NoAccount)?;
        let now = now_millis();
        let mut ids = Vec::new();
        for item in items {
            if !item.dfs_path.starts_with('/') || item.dfs_path.ends_with('/') {
                return Err(SdkError::Server(format!("上传路径格式不正确:{}", item.dfs_path)));
            }
            let (source_type, source) = match &item.source {
                UploadSource::File(path) => (SOURCE_FILE, path.clone()),
                UploadSource::Custom(id) => (SOURCE_CUSTOM, id.clone()),
            };
            let entry = UploadEntry {
                id: 0,
                account: account.name.clone(),
                source_type,
                source,
                dfs_path: item.dfs_path.clone(),
                size: None,
                uploaded: 0,
                md5: item.md5.clone().map(|it| it.to_ascii_lowercase()),
                content_type: item.content_type.clone(),
                thumb: item.thumb.clone(),
                skip_if_exists: item.skip_if_exists,
                state: STATE_WAITING,
                error: None,
                create_date: now,
                finish_date: None,
            };
            let id = self.inner.index.add_upload(&entry)?;
            self.inner.events.upload_state(id, STATE_WAITING, None);
            ids.push(id);
        }
        self.inner.uploads.schedule(&self.inner);
        Ok(ids)
    }

    pub fn get_upload(&self, id: i64) -> SdkResult<Option<UploadInfo>> {
        Ok(self.inner.index.get_upload(id)?.map(upload_info))
    }

    /// 按条件分页查询上传任务,按添加的顺序
    pub fn list_uploads(&self, query: &UploadQuery) -> SdkResult<Vec<UploadInfo>> {
        let list = self.inner.index.select_uploads(
            query.account.as_deref(),
            query.state.map(UploadState::to_i64),
            query.offset,
            query.limit,
        )?;
        Ok(list.into_iter().map(upload_info).collect())
    }

    /// 暂停上传
    pub fn pause_uploads(&self, ids: &[i64]) -> SdkResult<()> {
        for &id in ids {
            let Some(entry) = self.inner.index.get_upload(id)? else {
                continue;
            };
            if entry.state != STATE_WAITING && entry.state != STATE_DOWNLOADING {
                continue;
            }
            if !self.inner.uploads.stop(id, STATE_PAUSED) {
                self.inner.index.set_upload_state(id, STATE_PAUSED, None)?;
                self.inner.events.upload_state(id, STATE_PAUSED, None);
            }
        }
        Ok(())
    }

    /// 继续暂停或者失败的上传
    pub fn resume_uploads(&self, ids: &[i64]) -> SdkResult<()> {
        for &id in ids {
            let Some(entry) = self.inner.index.get_upload(id)? else {
                continue;
            };
            if entry.state != STATE_PAUSED && entry.state != STATE_ERROR {
                continue;
            }
            self.inner.index.set_upload_state(id, STATE_WAITING, None)?;
            self.inner.events.upload_state(id, STATE_WAITING, None);
        }
        self.inner.uploads.schedule(&self.inner);
        Ok(())
    }

    /// 暂停当前账号的所有上传
    pub fn pause_all_uploads(&self) -> SdkResult<()> {
        let account = self.inner.uploads.account().map(|it| it.name);
        let ids = self.upload_ids(account.as_deref(), &[STATE_WAITING, STATE_DOWNLOADING])?;
        self.pause_uploads(&ids)
    }

    /// 继续当前账号所有暂停或者失败的上传
    pub fn resume_all_uploads(&self) -> SdkResult<()> {
        let account = self.inner.uploads.account().map(|it| it.name);
        let ids = self.upload_ids(account.as_deref(), &[STATE_PAUSED, STATE_ERROR])?;
        self.resume_uploads(&ids)
    }

    /// 删除上传任务,正在上传的会先停止
    pub fn delete_uploads(&self, ids: &[i64]) -> SdkResult<()> {
        let mut removed = Vec::new();
        for &id in ids {
            //先删除记录再停止,停止之后调度时不会再次开始上传
            if self.inner.index.get_upload(id)?.is_some() {
                self.inner.index.delete_upload(id)?;
                removed.push(id);
            }
            if self.inner.uploads.stop(id, STATE_WAITING) {
                self.inner.rt.block_on(self.inner.uploads.wait_stopped(id));
            }
        }
        if !removed.is_empty() {
            self.inner.events.emit(Event::UploadRemoved { ids: removed });
        }
        Ok(())
    }

    /// 删除已经完成(包括跳过)的上传记录
    /// - account 账号,None时删除所有账号的
    pub fn clear_finished_uploads(&self, account: Option<&str>) -> SdkResult<()> {
        let ids = self.upload_ids(account, &[STATE_FINISH, STATE_SKIPPED])?;
        self.delete_uploads(&ids)
    }

    /// 修改同时上传的文件数
    pub fn set_max_uploading_count(&self, count: usize) {
        self.inner.config.lock().unwrap().max_uploading_count = count.max(1);
        self.inner.uploads.schedule(&self.inner);
    }

    fn upload_ids(&self, account: Option<&str>, states: &[i64]) -> SdkResult<Vec<i64>> {
        let mut ids = Vec::new();
        for &state in states {
            let list = self.inner.index.select_uploads(account, Some(state), 0, u32::MAX as u64)?;
            ids.extend(list.into_iter().map(|it| it.id));
        }
        Ok(ids)
    }
}

/// 启动时把上次没有完成的上传改成等待上传
pub(crate) fn init(inner: &Inner) -> SdkResult<()> {
    inner.index.change_upload_state(None, STATE_DOWNLOADING, STATE_WAITING)?;
    Ok(())
}

fn upload_info(it: UploadEntry) -> UploadInfo {
    UploadInfo {
        name: it.dfs_path.rsplit('/').next().unwrap_or("").to_string(),
        source: if it.source_type == SOURCE_CUSTOM { UploadSource::Custom(it.source) } else { UploadSource::File(it.source) },
        state: UploadState::from_i64(it.state),
        id: it.id,
        account: it.account,
        dfs_path: it.dfs_path,
        size: it.size,
        uploaded: it.uploaded,
        md5: it.md5,
        thumb: it.thumb,
        error: it.error,
        create_date: it.create_date,
        finish_date: it.finish_date,
    }
}
