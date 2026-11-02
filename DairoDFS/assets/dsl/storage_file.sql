-- 通过文件MD5获取本地存储文件
-- @name select_by_md5
-- @param md5:String
-- @return StorageFile?
select * from storage_file where md5 = :md5 limit 1;

-- ------------------------------

-- 获取没有被使用的存储文件ID
-- @name select_not_use_ids
-- @param time:i64 只获取这个时间之前创建的,避免正在上传的文件被回收
-- @return List<i64>
select id from storage_file
where id < :time
  and id not in (select storageId from dfs_file)
  and id not in (select storageId from dfs_file_delete)
  and md5 not in (select avatar from user where avatar is not null)
limit 1000;

-- ------------------------------

-- 统计没有被使用的存储文件数量(回收之前判断数量是否异常)
-- @name count_not_use
-- @param time:i64 只统计这个时间之前创建的
-- @return i64
select count(*) from storage_file
where id < :time
  and id not in (select storageId from dfs_file)
  and id not in (select storageId from dfs_file_delete)
  and md5 not in (select avatar from user where avatar is not null);

-- ------------------------------

-- 存储文件总数
-- @name count_all
-- @return i64
select count(*) from storage_file;

-- ------------------------------

-- 没有被使用时才删除存储记录(判断和删除在同一条语句里完成,避免判断之后又被其他请求使用)
-- @name delete_if_not_use
-- @param id:i64
delete from storage_file
where id = :id
  and not exists (select 1 from dfs_file where storageId = :id)
  and not exists (select 1 from dfs_file_delete where storageId = :id)
  and md5 not in (select avatar from user where avatar is not null);
