//! 缓存和下载的管理接口

use crate::cleaner;
use crate::download::Acquired;
use crate::error::{SdkError, SdkResult};
use crate::event::{DownloadState, Event, EventListener};
use crate::index::{Entry, STATE_DOWNLOADING, STATE_ERROR, STATE_FINISH, STATE_PAUSED, STATE_WAITING};
use crate::options::{CacheMode, Options};
use crate::{key, FileSdk};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 缓存信息
#[derive(Debug, Clone, PartialEq)]
pub struct CacheInfo {
    /// 缓存ID
    pub id: String,

    /// 最后一次使用的下载地址
    pub url: String,

    /// 命名空间
    pub namespace: String,

    /// 命名空间内的key
    pub key: String,

    /// 缓存分组
    pub group: String,

    /// 缓存方式
    pub mode: CacheMode,

    /// 文件名
    pub name: Option<String>,

    /// 缩略图地址
    pub thumb: Option<String>,

    /// 文件大小,未知时为None
    pub size: Option<u64>,

    /// 已经下载的大小
    pub downloaded: u64,

    /// 下载状态
    pub state: DownloadState,

    /// 下载失败的错误信息
    pub error: Option<String>,

    /// 创建时间(毫秒)
    pub create_date: i64,

    /// 下载完成时间(毫秒)
    pub finish_date: Option<i64>,

    /// 最后访问时间(毫秒)
    pub access_date: i64,

    /// 缓存文件路径,下载完成时才有
    pub path: Option<String>,
}

/// 分组统计
#[derive(Debug, Clone, PartialEq)]
pub struct GroupStats {
    pub group: String,

    /// 文件数
    pub count: u64,

    /// 已经下载的大小
    pub size: u64,
}

/// 查询条件
#[derive(Debug, Clone)]
pub struct CacheQuery {
    pub group: Option<String>,
    pub mode: Option<CacheMode>,
    pub state: Option<DownloadState>,
    pub offset: u64,
    pub limit: u64,
}

impl Default for CacheQuery {
    fn default() -> Self {
        Self { group: None, mode: None, state: None, offset: 0, limit: 100 }
    }
}

impl FileSdk {
    /// 设置事件回调,传None时取消
    pub fn set_listener(&self, listener: Option<Arc<dyn EventListener>>) {
        self.inner.events.set_listener(listener);
    }

    /// 获取下载地址对应的缓存ID,用于和事件、列表里的数据对应
    pub fn cache_id(&self, url: &str, options: &Options) -> SdkResult<String> {
        Ok(key::resolve(url, options)?.1.id)
    }

    /// 获取一条缓存信息
    pub fn get(&self, id: &str) -> SdkResult<Option<CacheInfo>> {
        Ok(self.inner.index.get(id)?.map(|it| self.info(it)))
    }

    /// 按条件分页查询缓存,按创建时间倒序
    pub fn list(&self, query: &CacheQuery) -> SdkResult<Vec<CacheInfo>> {
        let state = query.state.map(|it| match it {
            DownloadState::Waiting => STATE_WAITING,
            DownloadState::Downloading => STATE_DOWNLOADING,
            DownloadState::Paused => STATE_PAUSED,
            DownloadState::Error => STATE_ERROR,
            DownloadState::Finish => STATE_FINISH,
        });
        let list = self.inner.index.select(query.group.as_deref(), query.mode, state, query.offset, query.limit)?;
        Ok(list.into_iter().map(|it| self.info(it)).collect())
    }

    /// 按分组统计缓存数量和大小
    /// - mode 只统计指定的缓存方式,None时统计全部
    pub fn stats(&self, mode: Option<CacheMode>) -> SdkResult<Vec<GroupStats>> {
        let list = self.inner.index.stats(mode)?;
        Ok(list.into_iter().map(|(group, count, size)| GroupStats { group, count, size }).collect())
    }

    /// 删除缓存(包括永久保存的文件),正在下载的会先停止下载
    pub fn delete(&self, ids: &[String]) -> SdkResult<()> {
        let inner = &self.inner;
        let mut removed = Vec::new();
        for id in ids {
            inner.downloads.stop(inner, id, STATE_WAITING);

            //等待正在写入的任务结束之后再删除文件
            inner.rt.block_on(inner.downloads.wait_idle(id));
            if let Some(entry) = inner.index.get(id)? {
                cleaner::remove_files(inner, &entry);
                inner.index.delete(id)?;
                removed.push(id.clone());
            }
        }
        if !removed.is_empty() {
            inner.events.emit(Event::Removed { ids: removed });
        }
        Ok(())
    }

    /// 清理缓存(不包括永久保存的文件和正在使用的缓存),返回删除的数量
    /// - group 只清理指定分组,None时清理全部
    /// - before_access 只清理最后访问时间在这之前的(毫秒),None时不限制
    pub fn clear_cache(&self, group: Option<&str>, before_access: Option<i64>) -> SdkResult<u64> {
        let inner = &self.inner;
        inner.access.flush(&inner.index);
        let mut removed = Vec::new();
        for it in inner.index.select_cleanable(before_access.unwrap_or(i64::MAX))? {
            if group.is_some_and(|group| group != it.group) {
                continue;
            }
            if cleaner::remove_if_inactive(inner, &it)? {
                removed.push(it.id);
            }
        }
        let count = removed.len() as u64;
        if !removed.is_empty() {
            inner.events.emit(Event::Removed { ids: removed });
        }
        Ok(count)
    }

    /// 立即按照配置的保存天数和容量上限清理缓存,返回删除的数量
    pub fn clean(&self) -> SdkResult<u64> {
        cleaner::clean(&self.inner)
    }

    /// 添加永久保存的下载任务,不等待下载完成,返回缓存ID
    /// 下载进度和结果通过事件回调通知,App重启之后会继续下载
    pub fn download(&self, url: &str, options: &Options) -> SdkResult<String> {
        let options = Options { cache: CacheMode::Save, ..options.clone() };
        let id = key::resolve(url, &options)?.1.id;

        //永久保存的任务在没有请求等待时也会继续下载
        if let Acquired::Waiting(waiter) = self.inner.downloads.acquire(&self.inner, url, &options)? {
            drop(waiter);
        }
        Ok(id)
    }

    /// 暂停永久保存的下载任务
    pub fn pause(&self, ids: &[String]) -> SdkResult<()> {
        let inner = &self.inner;
        for id in ids {
            let Some(entry) = inner.index.get(id)? else {
                continue;
            };
            if entry.mode != CacheMode::Save || (entry.state != STATE_WAITING && entry.state != STATE_DOWNLOADING) {
                continue;
            }
            if !inner.downloads.stop(inner, id, STATE_PAUSED) {
                inner.index.set_state(id, STATE_PAUSED, None)?;
                inner.events.state(id, entry.mode, STATE_PAUSED, None);
            }
        }
        Ok(())
    }

    /// 继续暂停或者失败的永久保存下载任务
    pub fn resume(&self, ids: &[String]) -> SdkResult<()> {
        let inner = &self.inner;
        for id in ids {
            let Some(entry) = inner.index.get(id)? else {
                continue;
            };
            if entry.mode != CacheMode::Save || entry.state == STATE_FINISH || inner.downloads.resume_held(inner, id) {
                continue;
            }
            inner.index.set_state(id, STATE_WAITING, None)?;
            inner.events.state(id, entry.mode, STATE_WAITING, None);
            inner.downloads.resume_save(inner, &entry);
        }
        Ok(())
    }

    /// 暂停所有永久保存的下载任务
    pub fn pause_all(&self) -> SdkResult<()> {
        let ids = self.save_ids(&[STATE_WAITING, STATE_DOWNLOADING])?;
        self.pause(&ids)
    }

    /// 继续所有暂停或者失败的永久保存下载任务
    pub fn resume_all(&self) -> SdkResult<()> {
        let ids = self.save_ids(&[STATE_PAUSED, STATE_ERROR])?;
        self.resume(&ids)
    }

    /// 把下载完成的文件复制到指定文件夹,文件名重复时自动加上序号,返回复制后的文件路径
    /// - name 文件名,None时使用下载时指定的文件名
    pub fn export(&self, id: &str, dest_dir: &str, name: Option<&str>) -> SdkResult<String> {
        let Some(entry) = self.inner.index.get(id)? else {
            return Err(SdkError::NotFound);
        };
        let path = self.inner.root.join(entry.file());
        if entry.state != STATE_FINISH || !path.exists() {
            return Err(SdkError::NotFound);
        }
        let name = name.map(str::to_string).or(entry.name.clone()).unwrap_or_else(|| default_name(&entry));
        std::fs::create_dir_all(dest_dir)?;
        let target = unique_path(Path::new(dest_dir), &sanitize(&name));
        std::fs::copy(&path, &target)?;
        self.inner.access.record(&self.inner.index, id);
        Ok(target.to_string_lossy().to_string())
    }

    /// 修改同时缓存的文件数
    pub fn set_max_caching_count(&self, count: usize) {
        self.inner.config.lock().unwrap().max_caching_count = count.max(1);
        self.inner.downloads.schedule(&self.inner);
    }

    /// 修改同时永久保存(下载)的文件数
    pub fn set_max_saving_count(&self, count: usize) {
        self.inner.config.lock().unwrap().max_saving_count = count.max(1);
        self.inner.downloads.schedule(&self.inner);
    }

    /// 修改缓存保存天数,0表示不按时间清理
    pub fn set_cache_expire_days(&self, days: u32) {
        self.inner.config.lock().unwrap().cache_expire_days = days;
        cleaner::spawn_clean(&self.inner);
    }

    /// 修改缓存容量上限(字节),0表示不限制
    pub fn set_max_cache_size(&self, size: u64) {
        self.inner.config.lock().unwrap().max_cache_size = size;
        cleaner::spawn_clean(&self.inner);
    }

    /// 查询指定状态的永久保存任务
    fn save_ids(&self, states: &[i64]) -> SdkResult<Vec<String>> {
        let list = self.inner.index.select_by_mode(CacheMode::Save)?;
        Ok(list.into_iter().filter(|it| states.contains(&it.state)).map(|it| it.id).collect())
    }

    fn info(&self, it: Entry) -> CacheInfo {
        let path = self.inner.root.join(it.file());
        let path = (it.state == STATE_FINISH && path.exists()).then(|| path.to_string_lossy().to_string());
        CacheInfo {
            downloaded: it.ranges.len(),
            state: DownloadState::from_i64(it.state),
            path,
            id: it.id,
            url: it.url,
            namespace: it.namespace,
            key: it.key,
            group: it.group,
            mode: it.mode,
            name: it.name,
            thumb: it.thumb,
            size: it.size,
            error: it.error,
            create_date: it.create_date,
            finish_date: it.finish_date,
            access_date: it.access_date,
        }
    }
}

/// 没有指定文件名时,使用下载地址最后一段
fn default_name(entry: &Entry) -> String {
    let path = entry.url.split(['?', '#']).next().unwrap_or("");
    let name = path.rsplit('/').next().unwrap_or("");
    let name = percent_decode(name);
    if name.is_empty() {
        if entry.ext.is_empty() { entry.id.clone() } else { format!("{}.{}", entry.id, entry.ext) }
    } else {
        name
    }
}

/// 去掉文件名里不允许的字符
fn sanitize(name: &str) -> String {
    let name: String = name.chars().map(|it| if "/\\:*?\"<>|".contains(it) || it.is_control() { '_' } else { it }).collect();
    let name = name.trim().trim_matches('.').to_string();
    if name.is_empty() { "file".to_string() } else { name }
}

/// 文件已经存在时,在文件名后面加上序号:a.mp4 → a (1).mp4
fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    if !path.exists() {
        return path;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), format!(".{}", ext)),
        _ => (name.to_string(), String::new()),
    };
    (1..)
        .map(|i| dir.join(format!("{} ({}){}", stem, i, ext)))
        .find(|it| !it.exists())
        .unwrap()
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let (Some(h), Some(l)) = (bytes.get(i + 1).and_then(hex), bytes.get(i + 2).and_then(hex))
        {
            out.push(h * 16 + l);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex(b: &u8) -> Option<u8> {
    (*b as char).to_digit(16).map(|it| it as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(sanitize("a/b:c?.mp4"), "a_b_c_.mp4");
        assert_eq!(sanitize(".."), "file");
        assert_eq!(percent_decode("%E6%AD%8C.mp3"), "歌.mp3");
        assert_eq!(percent_decode("a%2"), "a%2");
        assert_eq!(percent_decode("%歌"), "%歌");
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("dairo-file-sdk-unique-{}-{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(unique_path(&dir, "a.mp4"), dir.join("a.mp4"));
        std::fs::write(dir.join("a.mp4"), "").unwrap();
        std::fs::write(dir.join("a (1).mp4"), "").unwrap();
        assert_eq!(unique_path(&dir, "a.mp4"), dir.join("a (2).mp4"));
        std::fs::write(dir.join("noext"), "").unwrap();
        assert_eq!(unique_path(&dir, "noext"), dir.join("noext (1)"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
