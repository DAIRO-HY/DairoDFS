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
limit 1000;
