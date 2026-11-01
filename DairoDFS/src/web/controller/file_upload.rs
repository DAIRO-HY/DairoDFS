use crate::application;
use crate::dao::storage_file_dao::StorageFile;
use crate::dao::{dfs_file_dao, storage_file_dao};
use crate::dao::dfs_file_dao::DfsFile;
use crate::service::{dfs_file_service, file_handle_service, storage_service};
use crate::util::file_util;
use crate::web::biz_error::BizError;
use crate::web::filter::auth::LoginUser;
use crate::{biz_error, db};
use hyperoute_core::{BodyStream, Ext, MultipartFile};
use hyperoute_macro::{GET, POST};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tokio::io::AsyncWriteExt;

/// 记录当前正在上传的文件MD5,避免同一个文件同时上传导致文件数据混乱
static UPLOADING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// 正在上传标记,离开作用域时自动移除
struct UploadingGuard(String);

impl UploadingGuard {
    fn new(md5: &str) -> Result<Self, BizError> {
        if !UPLOADING.lock().unwrap().insert(md5.to_string()) {
            return Err(biz_error!("文件服务繁忙，请稍后重试。"));
        }
        Ok(Self(md5.to_string()))
    }
}

impl Drop for UploadingGuard {
    fn drop(&mut self) {
        UPLOADING.lock().unwrap().remove(&self.0);
    }
}

/// 浏览器上传文件
/// - folder 上传到的文件夹
/// - content_type 文件类型,不指定时按扩展名判断
#[POST("/api/file_upload")]
pub async fn upload(
    login: Ext<LoginUser>,
    folder: String,
    content_type: Option<String>,
    file: MultipartFile<&Path>,
) -> Result<(), BizError> {
    let path = format!("{}/{}", folder, file.name);
    file_util::check_path(&path)?;
    let md5 = file_util::md5(file.body).await?;

    //上传的临时文件在请求结束后会被删除,这里直接移动到存储目录
    let storage = storage_service::save(file.body, &md5).await?;
    add_dfs_file(login.id, &storage, &path, content_type).await
}

/// 以流的方式上传文件,支持断点续传
/// 文件先追加到临时目录,全部上传完成并校验MD5后存放到存储目录,之后再调用by_md5添加到文件列表
/// - md5 文件的MD5
#[POST("/api/file_upload/by_stream/{md5}")]
pub async fn by_stream(md5: String, mut body: BodyStream) -> Result<(), BizError> {
    let temp_path = temp_path(&md5)?;
    let _guard = UploadingGuard::new(&md5)?;

    //以追加的方式打开文件
    let mut temp_file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
        .await?;
    while let Some(chunk) = body.next_chunk().await {
        let Ok(chunk) = chunk else {
            //可能是客户端中断,已经上传的部分保留,下次继续上传
            temp_file.flush().await?;
            return Err(biz_error!("上传中断"));
        };
        temp_file.write_all(&chunk).await?;
    }
    temp_file.flush().await?;
    drop(temp_file);

    //计算文件的MD5
    if file_util::md5(&temp_path).await? != md5 {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(biz_error!("文件校验失败"));
    }
    storage_service::save(&temp_path, &md5).await?;
    Ok(())
}

/// 获取文件已经上传的大小,断点续传时从这个位置开始上传
/// - md5 文件的MD5
#[GET("/api/file_upload/uploaded_size")]
pub async fn uploaded_size(md5: String) -> Result<u64, BizError> {
    let temp_path = temp_path(&md5)?;
    let _guard = UploadingGuard::new(&md5)?;
    let size = match tokio::fs::metadata(&temp_path).await {
        Ok(it) => it.len(),
        Err(_) => 0,
    };
    Ok(size)
}

/// 通过MD5上传(文件已经存在于存储目录时,只需要添加到文件列表)
/// - md5 文件的MD5
/// - path 文件路径
/// - content_type 文件类型,不指定时按扩展名判断
#[POST("/api/file_upload/by_md5")]
pub async fn by_md5(login: Ext<LoginUser>, md5: String, path: String, content_type: Option<String>) -> Result<(), BizError> {
    let temp_path = temp_path(&md5)?;
    let Some(storage) = storage_file_dao::select_by_md5(&mut lib_db::get_context(), &md5).await? else {
        //文件上传中发生了意外,删除临时文件重新上传
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(biz_error!("文件夹或文件不存在"));
    };
    add_dfs_file(login.id, &storage, &path, content_type).await?;
    let _ = tokio::fs::remove_file(&temp_path).await;
    Ok(())
}

/// 检查自己的文件里是否已经存在该文件
/// - md5 文件的MD5
#[GET("/api/file_upload/exists")]
pub async fn exists(login: Ext<LoginUser>, md5: String) -> Result<bool, BizError> {
    let count = dfs_file_dao::count_by_md5(&mut lib_db::get_context(), login.id, &md5).await?;
    Ok(count > 0)
}

/// 断点续传的临时文件路径,MD5会用作文件名,必须验证格式
fn temp_path(md5: &str) -> Result<PathBuf, BizError> {
    if md5.len() != 32 || !md5.bytes().all(|it| it.is_ascii_digit() || (b'a'..=b'f').contains(&it)) {
        return Err(biz_error!("MD5格式不正确,必须是32位小写16进制字符"));
    }
    Ok(application::TEMP_PATH.join(md5))
}

/// 添加到文件列表
/// - path 文件路径
/// - content_type 文件类型,不指定时按扩展名判断
async fn add_dfs_file(user_id: i64, storage: &StorageFile, path: &str, content_type: Option<String>) -> Result<(), BizError> {
    let name = file_util::file_name(path);
    let content_type = match content_type.filter(|it| !it.is_empty()) {
        Some(it) => it,
        None => file_util::content_type(&file_util::file_ext(name)),
    };
    let size = tokio::fs::metadata(&storage.path).await?.len() as i64;
    let mut ctx = db::begin().await?;
    let folder_id = dfs_file_service::get_folder_id(&mut ctx, user_id, file_util::file_parent(path), true)
        .await?
        .unwrap();
    let file = DfsFile {
        userId: user_id,
        parentId: folder_id,
        name: name.to_string(),
        size,
        contentType: Some(content_type),
        storageId: storage.id,
        ..Default::default()
    };
    dfs_file_service::add_file(&mut ctx, file, true).await?;
    db::commit(ctx).await?;

    //生成缩略图等
    file_handle_service::notify();
    Ok(())
}
