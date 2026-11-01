//! 自动清理缓存:超过保存天数没有访问的,以及超过容量上限时最久没有访问的
//! 永久保存的文件不会被清理,正在下载或者正在被读取的文件也不会被清理

use crate::download::part_path;
use crate::error::SdkResult;
use crate::event::Event;
use crate::index::Entry;
use crate::{now_millis, Inner};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 定时清理的间隔
pub const CLEAN_INTERVAL: Duration = Duration::from_secs(10 * 60);

/// 下载完成之后检查容量的最短间隔,避免下载大量小文件时频繁查询
const CHECK_SIZE_INTERVAL: Duration = Duration::from_secs(5);

const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

#[derive(Default)]
pub(crate) struct Cleaner {
    /// 正在清理,同时只能有一个清理任务
    running: AtomicBool,

    /// 上次检查容量的时间
    last_check: Mutex<Option<Instant>>,
}

/// 下载完成之后检查是否超过容量上限,超过时在后台清理
pub(crate) fn check_size(inner: &Arc<Inner>) {
    let max = inner.config().max_cache_size;
    if max == 0 {
        return;
    }
    {
        let mut last_check = inner.cleaner.last_check.lock().unwrap();
        if last_check.is_some_and(|it| it.elapsed() < CHECK_SIZE_INTERVAL) {
            return;
        }
        *last_check = Some(Instant::now());
    }
    if inner.index.cleanable_size().unwrap_or(0) > max {
        spawn_clean(inner);
    }
}

/// 在后台清理
pub(crate) fn spawn_clean(inner: &Arc<Inner>) {
    let inner = inner.clone();
    inner.rt.clone().spawn_blocking(move || {
        if let Err(e) = clean(&inner) {
            eprintln!("-->dairo-file-sdk:清理缓存失败:{}", e);
        }
    });
}

/// 按照配置清理缓存,返回被删除的数量
pub(crate) fn clean(inner: &Arc<Inner>) -> SdkResult<u64> {
    if inner.cleaner.running.swap(true, Ordering::AcqRel) {
        return Ok(0);
    }
    let result = clean_inner(inner);
    inner.cleaner.running.store(false, Ordering::Release);
    result
}

fn clean_inner(inner: &Arc<Inner>) -> SdkResult<u64> {
    //先把内存里的访问时间写入数据库,避免刚访问过的文件被清理
    inner.access.flush(&inner.index);
    let config = inner.config();
    let mut removed = Vec::new();
    if config.cache_expire_days > 0 {
        let before = now_millis() - config.cache_expire_days as i64 * DAY_MILLIS;
        for it in inner.index.select_cleanable(before)? {
            if remove_if_inactive(inner, &it)? {
                removed.push(it.id);
            }
        }
    }
    if config.max_cache_size > 0 {
        let mut total = inner.index.cleanable_size()?;
        if total > config.max_cache_size {
            for it in inner.index.select_cleanable(i64::MAX)? {
                if total <= config.max_cache_size {
                    break;
                }
                if remove_if_inactive(inner, &it)? {
                    total = total.saturating_sub(it.ranges.len());
                    removed.push(it.id);
                }
            }
        }
    }
    let count = removed.len() as u64;
    if !removed.is_empty() {
        inner.events.emit(Event::Removed { ids: removed });
    }
    Ok(count)
}

/// 没有在下载时删除缓存
pub(crate) fn remove_if_inactive(inner: &Arc<Inner>, entry: &Entry) -> SdkResult<bool> {
    inner.downloads.with_inactive(&entry.id, || {
        remove_files(inner, entry);
        inner.index.delete(&entry.id)
    })
}

/// 删除缓存文件和下载中的临时文件
pub(crate) fn remove_files(inner: &Inner, entry: &Entry) {
    let path = inner.root.join(entry.file());
    let _ = std::fs::remove_file(part_path(&path));
    let _ = std::fs::remove_file(path);
}
