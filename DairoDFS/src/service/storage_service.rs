use crate::dao::storage_file_dao;
use crate::dao::storage_file_dao::StorageFile;
use crate::util::{id_util, time_util};
use crate::web::biz_error::BizError;
use crate::{biz_error, biz_errorf, db, system_config};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 记录上一次生成的存储文件名,避免并发时生成重复的文件名
static PRE_FILE_NAME: Mutex<i64> = Mutex::new(0);

/// 将已经保存在临时目录的文件存放到存储目录
/// 同一个文件(MD5相同)只存储一份,已经存在时直接删除临时文件
/// - temp_path 临时文件路径,无论成功与否,调用之后该文件都不再存在
/// - md5 文件的MD5
pub async fn save(temp_path: &Path, md5: &str) -> Result<StorageFile, BizError> {
    let result = save_inner(temp_path, md5).await;
    if tokio::fs::try_exists(temp_path).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(temp_path).await;
    }
    result
}

async fn save_inner(temp_path: &Path, md5: &str) -> Result<StorageFile, BizError> {
    if let Some(it) = storage_file_dao::select_by_md5(&mut lib_db::get_context(), md5).await? {
        //该文件已经存在
        return Ok(it);
    }
    let size = tokio::fs::metadata(temp_path).await?.len();
    let local_path = make_local_path(size).await?;

    //临时目录和存储目录在同一块磁盘上时直接改名,否则复制
    if tokio::fs::rename(temp_path, &local_path).await.is_err() {
        if let Err(e) = tokio::fs::copy(temp_path, &local_path).await {
            let _ = tokio::fs::remove_file(&local_path).await;
            return Err(biz_errorf!("保存文件失败：{}", e));
        }
    }

    //确保文件已经写入到了磁盘，避免突然断电导致文件数据丢失
    tokio::fs::File::open(&local_path).await?.sync_all().await?;
    let storage_file = StorageFile {
        id: id_util::id(),
        path: local_path.to_string_lossy().to_string(),
        md5: md5.to_string(),
    };
    let mut ctx = db::begin().await?;
    storage_file_dao::insert(&mut ctx, &storage_file).await?;
    db::commit(ctx).await?;
    Ok(storage_file)
}

/// 将文件移动到存储目录(不添加存储记录),返回存储路径
/// 临时目录和存储目录在同一块磁盘上时直接改名,否则复制
pub async fn move_to_storage(temp_path: &Path) -> Result<PathBuf, BizError> {
    let size = tokio::fs::metadata(temp_path).await?.len();
    let local_path = make_local_path(size).await?;
    if tokio::fs::rename(temp_path, &local_path).await.is_err() {
        if let Err(e) = tokio::fs::copy(temp_path, &local_path).await {
            let _ = tokio::fs::remove_file(&local_path).await;
            return Err(biz_errorf!("保存文件失败：{}", e));
        }
        let _ = tokio::fs::remove_file(temp_path).await;
    }
    tokio::fs::File::open(&local_path).await?.sync_all().await?;
    Ok(local_path)
}

/// 生成本地文件存储路径: 存储目录/年-月/微秒时间戳
async fn make_local_path(size: u64) -> Result<PathBuf, BizError> {
    let folder = select_driver_folder(size)?.join(chrono::Local::now().format("%Y-%m").to_string());
    tokio::fs::create_dir_all(&folder).await.map_err(|e| biz_errorf!("创建文件夹失败：{}", e))?;
    let file_name = {
        let mut pre = PRE_FILE_NAME.lock().unwrap();
        let file_name = time_util::current_micros().max(*pre + 1);
        *pre = file_name;
        file_name
    };
    let path = folder.join(file_name.to_string());
    if tokio::fs::try_exists(&path).await.unwrap_or(false) {
        //文件已经存在(小概率事件)
        return Err(biz_error!("准备创建的文件已经存在"));
    }
    Ok(path)
}

/// 判断存储目录的磁盘剩余容量,选择合适的目录
fn select_driver_folder(size: u64) -> Result<PathBuf, BizError> {
    let config = system_config::get();
    if config.save_folder_list.is_empty() {
        return Err(biz_error!("没有配置存储目录"));
    }
    for folder in &config.save_folder_list {
        let path = Path::new(folder);
        if !path.is_dir() {
            continue;
        }
        if fs4::available_space(path).is_ok_and(|it| it > size) {
            return Ok(path.to_path_buf());
        }
    }
    Err(biz_error!("设置的存储文件夹不存在或没有足够存储空间"))
}

/// 将数据存放到存储目录
pub async fn save_data(data: &[u8]) -> Result<StorageFile, BizError> {
    let md5 = format!("{:x}", md5::compute(data));
    let temp_path = crate::util::media::temp_file_path();
    tokio::fs::write(&temp_path, data).await?;
    save(&temp_path, &md5).await
}
