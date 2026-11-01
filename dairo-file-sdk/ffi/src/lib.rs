//! dairo-file-sdk的Kotlin/Swift绑定
//!
//! 方法都是同步的,会读写数据库或者等待任务停止的方法(delete、export等)不要在主线程调用;
//! 下载文件用fetch(回调)或者fetchAsync(Kotlin协程/Swift async)

mod callback;
mod types;

pub use callback::{EventListener, FetchListener, UploadSourceProvider};
pub use types::*;

use callback::{EventListenerAdapter, FetchListenerAdapter, UploadSourceProviderAdapter};
use dairo_file_sdk as core;
use std::sync::Arc;

uniffi::setup_scaffolding!();

type Result<T> = std::result::Result<T, SdkError>;

/// 一次下载请求,可以取消
#[derive(uniffi::Object)]
pub struct FetchTask(core::FetchTask);

#[uniffi::export]
impl FetchTask {
    /// 取消请求,没有其他请求在等待同一个文件时会停止下载(永久保存的任务不会停止),取消之后不会再回调
    pub fn cancel(&self) {
        self.0.cancel();
    }
}

/// 文件SDK,一个数据目录只能创建一个实例,建议整个App共用一个
#[derive(uniffi::Object)]
pub struct FileSdk(core::FileSdk);

#[uniffi::export]
impl FileSdk {
    #[uniffi::constructor]
    pub fn new(config: SdkConfig) -> Result<Arc<Self>> {
        Ok(Arc::new(Self(core::FileSdk::new(config.into())?)))
    }

    /*----------------------------------------下载和缓存----------------------------------------*/

    /// 下载文件,完成之后通过回调返回缓存文件路径(已经缓存时立即回调)
    pub fn fetch(&self, url: String, options: Options, listener: Arc<dyn FetchListener>) -> Arc<FetchTask> {
        Arc::new(FetchTask(self.0.fetch(&url, options.into(), Arc::new(FetchListenerAdapter(listener)))))
    }

    /// 下载文件,完成之后返回缓存文件路径
    pub async fn fetch_async(&self, url: String, options: Options) -> Result<String> {
        Ok(self.0.fetch_async(&url, options.into()).await?)
    }

    /// 获取已经缓存的文件路径,没有缓存时返回null
    pub fn cached_path(&self, url: String, options: Options) -> Result<Option<String>> {
        Ok(self.0.cached_path(&url, &options.into())?)
    }

    /// 把本地文件复制到缓存,之后可以像下载过的文件一样访问,返回缓存文件路径
    pub fn put_file(&self, url: String, options: Options, path: String) -> Result<String> {
        Ok(self.0.put_file(&url, &options.into(), core::PutSource::Path(path))?)
    }

    /// 把数据存入缓存,返回缓存文件路径
    pub fn put_data(&self, url: String, options: Options, data: Vec<u8>) -> Result<String> {
        Ok(self.0.put_file(&url, &options.into(), core::PutSource::Bytes(data))?)
    }

    /// 把内存中记录的访问时间写入数据库,App进入后台时调用
    pub fn flush(&self) {
        self.0.flush();
    }

    /*----------------------------------------本机HTTP----------------------------------------*/

    /// 登记一个下载地址,返回本机HTTP访问地址,交给播放器、图片组件等使用(支持Range,边下边播)
    pub fn open(&self, url: String, options: Options) -> Result<String> {
        Ok(self.0.open(&url, &options.into())?)
    }

    /// 生成无状态的本机HTTP访问地址(参数和下载地址编码在地址里),适合列表里大量的缩略图
    pub fn local_url(&self, url: String, options: Options) -> Result<String> {
        Ok(self.0.local_url(&url, &options.into())?)
    }

    /// 本机HTTP访问地址的前缀
    pub fn local_base(&self) -> Result<String> {
        Ok(self.0.local_base()?)
    }

    /// 检查本机HTTP服务,不能访问时重新启动,返回监听的端口(iOS从后台回到前台时调用)
    pub fn check_server(&self) -> Result<u16> {
        Ok(self.0.check_server()?)
    }

    /// 停止本机HTTP服务,下次访问时会自动启动
    pub fn stop_server(&self) {
        self.0.stop_server();
    }

    /*----------------------------------------管理----------------------------------------*/

    /// 设置事件回调,传null时取消
    pub fn set_listener(&self, listener: Option<Arc<dyn EventListener>>) {
        self.0.set_listener(listener.map(|it| Arc::new(EventListenerAdapter(it)) as Arc<dyn core::EventListener>));
    }

    /// 获取下载地址对应的缓存ID,用于和事件、列表里的数据对应
    pub fn cache_id(&self, url: String, options: Options) -> Result<String> {
        Ok(self.0.cache_id(&url, &options.into())?)
    }

    /// 获取一条缓存信息
    pub fn get(&self, id: String) -> Result<Option<CacheInfo>> {
        Ok(self.0.get(&id)?.map(Into::into))
    }

    /// 按条件分页查询缓存,按创建时间倒序
    pub fn list(&self, query: CacheQuery) -> Result<Vec<CacheInfo>> {
        Ok(self.0.list(&query.into())?.into_iter().map(Into::into).collect())
    }

    /// 按分组统计缓存数量和大小,mode为null时统计全部
    pub fn stats(&self, mode: Option<CacheMode>) -> Result<Vec<GroupStats>> {
        let list = self.0.stats(mode.map(Into::into))?;
        Ok(list.into_iter().map(|it| GroupStats { group: it.group, count: it.count, size: it.size }).collect())
    }

    /// 删除缓存(包括永久保存的文件),正在下载的会先停止下载
    pub fn delete(&self, ids: Vec<String>) -> Result<()> {
        Ok(self.0.delete(&ids)?)
    }

    /// 清理缓存(不包括永久保存的文件和正在使用的缓存),返回删除的数量
    /// - group 只清理指定分组,null时清理全部
    /// - before_access 只清理最后访问时间在这之前的(毫秒),null时不限制
    pub fn clear_cache(&self, group: Option<String>, before_access: Option<i64>) -> Result<u64> {
        Ok(self.0.clear_cache(group.as_deref(), before_access)?)
    }

    /// 立即按照配置的保存天数和容量上限清理缓存,返回删除的数量
    pub fn clean(&self) -> Result<u64> {
        Ok(self.0.clean()?)
    }

    /// 添加永久保存的下载任务,不等待下载完成,返回缓存ID
    pub fn download(&self, url: String, options: Options) -> Result<String> {
        Ok(self.0.download(&url, &options.into())?)
    }

    /// 暂停永久保存的下载任务
    pub fn pause(&self, ids: Vec<String>) -> Result<()> {
        Ok(self.0.pause(&ids)?)
    }

    /// 继续暂停或者失败的永久保存下载任务
    pub fn resume(&self, ids: Vec<String>) -> Result<()> {
        Ok(self.0.resume(&ids)?)
    }

    /// 暂停所有永久保存的下载任务
    pub fn pause_all(&self) -> Result<()> {
        Ok(self.0.pause_all()?)
    }

    /// 继续所有暂停或者失败的永久保存下载任务
    pub fn resume_all(&self) -> Result<()> {
        Ok(self.0.resume_all()?)
    }

    /// 把下载完成的文件复制到指定文件夹,文件名重复时自动加上序号,返回复制后的文件路径
    pub fn export(&self, id: String, dest_dir: String, name: Option<String>) -> Result<String> {
        Ok(self.0.export(&id, &dest_dir, name.as_deref())?)
    }

    pub fn set_max_caching_count(&self, count: u32) {
        self.0.set_max_caching_count(count as usize);
    }

    pub fn set_max_saving_count(&self, count: u32) {
        self.0.set_max_saving_count(count as usize);
    }

    /// 缓存保存天数,0表示不按时间清理
    pub fn set_cache_expire_days(&self, days: u32) {
        self.0.set_cache_expire_days(days);
    }

    /// 缓存容量上限(字节),0表示不限制
    pub fn set_max_cache_size(&self, size: u64) {
        self.0.set_max_cache_size(size);
    }

    /*----------------------------------------上传到DFS----------------------------------------*/

    /// 设置当前登录的DFS账号,只有这个账号的任务会上传
    /// - account 账号标识,由平台定义,建议使用"集群ID:用户ID"
    /// - server DFS服务器地址,例如:http://192.168.1.2:8031
    /// - token 登录票据
    pub fn set_dfs_account(&self, account: String, server: String, token: String) {
        self.0.set_dfs_account(&account, &server, &token);
    }

    /// 退出登录,停止所有上传,任务保留
    pub fn clear_dfs_account(&self) {
        self.0.clear_dfs_account();
    }

    /// 设置平台提供上传数据的回调
    pub fn set_upload_source_provider(&self, provider: Option<Arc<dyn UploadSourceProvider>>) {
        self.0.set_upload_source_provider(
            provider.map(|it| Arc::new(UploadSourceProviderAdapter(it)) as Arc<dyn core::UploadSourceProvider>),
        );
    }

    /// 添加上传任务到当前账号,返回上传ID
    pub fn add_uploads(&self, items: Vec<UploadItem>) -> Result<Vec<i64>> {
        let items: Vec<core::UploadItem> = items.into_iter().map(Into::into).collect();
        Ok(self.0.add_uploads(&items)?)
    }

    pub fn get_upload(&self, id: i64) -> Result<Option<UploadInfo>> {
        Ok(self.0.get_upload(id)?.map(Into::into))
    }

    /// 按条件分页查询上传任务,按添加的顺序
    pub fn list_uploads(&self, query: UploadQuery) -> Result<Vec<UploadInfo>> {
        Ok(self.0.list_uploads(&query.into())?.into_iter().map(Into::into).collect())
    }

    pub fn pause_uploads(&self, ids: Vec<i64>) -> Result<()> {
        Ok(self.0.pause_uploads(&ids)?)
    }

    pub fn resume_uploads(&self, ids: Vec<i64>) -> Result<()> {
        Ok(self.0.resume_uploads(&ids)?)
    }

    pub fn pause_all_uploads(&self) -> Result<()> {
        Ok(self.0.pause_all_uploads()?)
    }

    pub fn resume_all_uploads(&self) -> Result<()> {
        Ok(self.0.resume_all_uploads()?)
    }

    /// 删除上传任务,正在上传的会先停止
    pub fn delete_uploads(&self, ids: Vec<i64>) -> Result<()> {
        Ok(self.0.delete_uploads(&ids)?)
    }

    /// 删除已经完成(包括跳过)的上传记录,account为null时删除所有账号的
    pub fn clear_finished_uploads(&self, account: Option<String>) -> Result<()> {
        Ok(self.0.clear_finished_uploads(account.as_deref())?)
    }

    pub fn set_max_uploading_count(&self, count: u32) {
        self.0.set_max_uploading_count(count as usize);
    }
}
