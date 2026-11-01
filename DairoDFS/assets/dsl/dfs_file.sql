-- 通过文件夹ID和文件名获取当前版本未删除的文件(文件名比较时忽略大小写)
-- @name select_by_parent_id_and_name
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @return DfsFile?
select * from dfs_file
where userId = :user_id
  and parentId = :parent_id
  and name = :name COLLATE NOCASE
  and isHistory = 0
  and deleteDate is null;

-- ------------------------------

-- 获取当前版本未删除的子文件及子文件夹
-- @name select_sub_list
-- @param user_id:i64
-- @param parent_id:i64
-- @return List<DfsFile>
select * from dfs_file
where userId = :user_id
  and parentId = :parent_id
  and isHistory = 0
  and deleteDate is null
order by storageId > 0, date;

-- ------------------------------

-- 获取有缩略图的文件ID(文件列表显示缩略图用)
-- @name select_has_thumb_ids
-- @param user_id:i64
-- @param parent_id:i64
-- @return List<i64>
select df.id from dfs_file as df
where df.userId = :user_id
  and df.parentId = :parent_id
  and df.storageId > 0
  and df.isHistory = 0
  and df.deleteDate is null
  and exists(select 1 from dfs_file as thumbDf where thumbDf.parentId = df.id and thumbDf.isExtra = 1 and thumbDf.name = 'thumb');

-- ------------------------------

-- 获取文件夹下的所有记录,包括历史文件和已删除文件
-- @name select_all_child_list
-- @param parent_id:i64
-- @return List<DfsFile>
select * from dfs_file where parentId = :parent_id;

-- ------------------------------

-- 获取某个文件的附属文件
-- @name select_extra_list
-- @param id:i64
-- @return List<DfsFile>
select * from dfs_file where parentId = :id and isExtra = 1;

-- ------------------------------

-- 获取某个文件指定名称的附属文件
-- @name select_extra
-- @param id:i64
-- @param name:String
-- @return DfsFile?
select * from dfs_file where parentId = :id and name = :name and isExtra = 1;

-- ------------------------------

-- 获取文件的历史版本
-- @name select_history
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @return List<DfsFile>
select * from dfs_file
where userId = :user_id
  and parentId = :parent_id
  and name = :name
  and storageId > 0
  and isHistory = 1
  and deleteDate is null
order by date desc;

-- ------------------------------

-- 获取垃圾箱里的文件(直接被删除的文件和文件夹)
-- @name select_delete
-- @param user_id:i64
-- @return List<DfsFile>
select * from dfs_file
where userId = :user_id
  and isHistory = 0
  and isExtra = 0
  and deleteDate is not null
order by deleteDate desc;

-- ------------------------------

-- 获取同一批被删除的记录(被删除的文件及其历史版本)
-- @name select_same_delete
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @param delete_date:i64
-- @return List<DfsFile>
select * from dfs_file
where userId = :user_id
  and parentId = :parent_id
  and name = :name
  and deleteDate = :delete_date;

-- ------------------------------

-- 将文件标记为历史版本
-- @name set_history
-- @param id:i64
update dfs_file set isHistory = 1 where id = :id;

-- ------------------------------

-- 将文件及其历史版本标记为删除
-- @name set_delete
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @param time:i64
update dfs_file set deleteDate = :time
where userId = :user_id
  and parentId = :parent_id
  and name = :name
  and deleteDate is null;

-- ------------------------------

-- 将同一批被删除的文件及其历史版本还原
-- @name set_not_delete
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @param delete_date:i64
update dfs_file set deleteDate = null
where userId = :user_id
  and parentId = :parent_id
  and name = :name
  and deleteDate = :delete_date;

-- ------------------------------

-- 修改文件类型
-- @name set_content_type
-- @param id:i64
-- @param content_type:String
update dfs_file set contentType = :content_type where id = :id and storageId > 0;

-- ------------------------------

-- 移动或重命名文件夹(文件夹没有历史版本,只修改它自己,子文件跟着一起移动)
-- @name set_parent_and_name
-- @param id:i64
-- @param parent_id:i64
-- @param name:String
update dfs_file set parentId = :parent_id, name = :name where id = :id;

-- ------------------------------

-- 移动或重命名文件,历史版本一起移动,否则历史版本会和文件脱离
-- @name move_file_with_history
-- @param user_id:i64
-- @param parent_id:i64
-- @param name:String
-- @param target_parent_id:i64
-- @param target_name:String
update dfs_file set parentId = :target_parent_id, name = :target_name
where userId = :user_id
  and parentId = :parent_id
  and name = :name
  and storageId > 0
  and deleteDate is null;

-- ------------------------------

-- 某个用户是否已经存在指定MD5的文件
-- @name count_by_md5
-- @param user_id:i64
-- @param md5:String
-- @return i64
select count(*) from dfs_file
where storageId = (select id from storage_file where md5 = :md5)
  and userId = :user_id
  and isHistory = 0
  and isExtra = 0
  and deleteDate is null;

-- ------------------------------

-- 本地存储文件是否还在被使用
-- @name count_by_storage_id
-- @param storage_id:i64
-- @return i64
select count(*) from dfs_file where storageId = :storage_id;

-- ------------------------------

-- 获取尚未处理的文件(生成属性、缩略图等)
-- @name select_no_handle
-- @return List<DfsFile>
select * from dfs_file where storageId > 0 and isExtra = 0 and state = 0 order by id limit 100;

-- ------------------------------

-- 通过本地存储ID获取已经生成的文件属性(同一个文件只需要生成一次)
-- @name select_property_by_storage_id
-- @param storage_id:i64
-- @return String?
select property from dfs_file where storageId = :storage_id and isExtra = 0 and state = 1 and property is not null limit 1;

-- ------------------------------

-- 通过本地存储ID和附属文件名获取其他文件已经生成的附属文件(同一个文件只需要生成一次)
-- @name select_extra_by_storage_id_and_name
-- @param storage_id:i64
-- @param name:String
-- @return DfsFile?
select * from dfs_file
where parentId = (select id from dfs_file where storageId = :storage_id and isExtra = 0 and state = 1 limit 1)
  and name = :name
  and isExtra = 1
limit 1;

-- ------------------------------

-- 设置文件属性
-- @name set_property
-- @param id:i64
-- @param property:String
update dfs_file set property = :property where id = :id;

-- ------------------------------

-- 设置文件处理状态
-- @name set_state
-- @param id:i64
-- @param state:i64
-- @param state_msg:String?
update dfs_file set state = :state, stateMsg = :state_msg where id = :id;

-- ------------------------------

-- 获取相册文件(图片和视频)
-- @name select_album
-- @param user_id:i64
-- @return List<DfsFile>
select * from dfs_file
where userId = :user_id
  and ext in ('jpg','jpeg','png','bmp','gif','ico','svg','tiff','webp','wmf','wmz','jp2','eps','tga','jfif','psd','psb','ai','mp4','mov','avi','mkv','flv','rm','rmvb','3gp','cr3','cr2','heic','hif','dlive')
  and storageId > 0
  and isExtra = 0
  and isHistory = 0
  and deleteDate is null;

-- ------------------------------

-- 获取某个用户所有有缩略图的文件ID
-- @name select_thumb_parent_ids
-- @param user_id:i64
-- @return List<i64>
select parentId from dfs_file where userId = :user_id and isExtra = 1 and name = 'thumb';

-- ------------------------------

-- 获取垃圾箱里超时的文件ID
-- @name select_ids_by_delete_timeout
-- @param time:i64
-- @return List<i64>
select id from dfs_file where deleteDate < :time limit 1000;
