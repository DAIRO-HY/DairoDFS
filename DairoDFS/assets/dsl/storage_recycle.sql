-- 获取移入隔离区超过指定时间的记录
-- @name select_before
-- @param time:i64
-- @return List<StorageRecycle>
select * from storage_recycle where date < :time order by date limit 1000;

-- ------------------------------

-- 获取隔离区里仍然被使用的记录(说明被误删,需要恢复)
-- @name select_used
-- @return List<StorageRecycle>
select * from storage_recycle
where id in (select storageId from dfs_file)
   or id in (select storageId from dfs_file_delete)
   or md5 in (select avatar from user where avatar is not null);

-- ------------------------------

-- 隔离区统计
-- @name stats
-- @return StorageRecycleStats
select count(*) as count, coalesce(sum(size), 0) as size, min(date) as minDate from storage_recycle;
