//! 由平台实现的回调,转接成核心库的回调

use crate::types::{CallbackError, Event, SdkError};
use dairo_file_sdk as core;
use std::sync::Arc;

/// 下载结果回调,在SDK的后台线程里调用
#[uniffi::export(with_foreign)]
pub trait FetchListener: Send + Sync {
    /// 下载进度,size为文件大小,未知时为null
    fn on_progress(&self, size: Option<u64>, downloaded: u64);

    /// 下载完成,path为缓存文件路径
    fn on_success(&self, path: String);

    /// 下载失败
    fn on_error(&self, error: SdkError);
}

/// 事件回调,在SDK的事件线程里按顺序调用,不要在回调里做耗时的操作
#[uniffi::export(with_foreign)]
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: Event);
}

/// 平台提供上传数据,用于无法直接通过文件路径读取的数据(iOS相册、实况照片、Android的content://等)
/// 在SDK的后台线程里调用,可以在这里做耗时的操作
#[uniffi::export(with_foreign)]
pub trait UploadSourceProvider: Send + Sync {
    /// 数据的总大小
    fn size(&self, id: String) -> Result<u64, CallbackError>;

    /// 读取数据,返回的数据长度必须等于len(最后一段除外)
    fn read(&self, id: String, offset: u64, len: u64) -> Result<Vec<u8>, CallbackError>;

    /// 读取结束,可以在这里释放资源
    fn close(&self, id: String);
}

pub(crate) struct FetchListenerAdapter(pub Arc<dyn FetchListener>);

impl core::FetchListener for FetchListenerAdapter {
    fn on_progress(&self, size: Option<u64>, downloaded: u64) {
        self.0.on_progress(size, downloaded);
    }

    fn on_success(&self, path: String) {
        self.0.on_success(path);
    }

    fn on_error(&self, error: core::SdkError) {
        self.0.on_error(error.into());
    }
}

pub(crate) struct EventListenerAdapter(pub Arc<dyn EventListener>);

impl core::EventListener for EventListenerAdapter {
    fn on_event(&self, event: core::Event) {
        self.0.on_event(event.into());
    }
}

pub(crate) struct UploadSourceProviderAdapter(pub Arc<dyn UploadSourceProvider>);

impl core::UploadSourceProvider for UploadSourceProviderAdapter {
    fn size(&self, source: String) -> Result<u64, String> {
        self.0.size(source).map_err(|e| e.to_string())
    }

    fn read(&self, source: String, offset: u64, len: u64) -> Result<Vec<u8>, String> {
        self.0.read(source, offset, len).map_err(|e| e.to_string())
    }

    fn close(&self, source: String) {
        self.0.close(source);
    }
}
