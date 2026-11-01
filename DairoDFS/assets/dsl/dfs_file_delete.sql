-- 将文件记录复制到彻底删除表,并设置删除时间(之后由定时任务回收本地存储文件)
-- @name insert_from_dfs_file
-- @param id:i64
-- @param time:i64
insert into dfs_file_delete(id, userId, parentId, name, ext, size, contentType, storageId, date, property, isExtra, isHistory, deleteDate, state, stateMsg)
select id, userId, parentId, name, ext, size, contentType, storageId, date, property, isExtra, isHistory, :time, state, stateMsg
from dfs_file where id = :id;

-- ------------------------------

-- 本地存储文件是否还在被使用
-- @name count_by_storage_id
-- @param storage_id:i64
-- @return i64
select count(*) from dfs_file_delete where storageId = :storage_id;

-- ------------------------------

-- 获取彻底删除超时的记录(之后回收本地存储文件)
-- @name select_by_timeout
-- @param time:i64
-- @return List<DfsFileDelete>
select * from dfs_file_delete where deleteDate < :time limit 1000;
