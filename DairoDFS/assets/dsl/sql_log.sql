-- 获取当前日志中最大的ID
-- @name select_last_id
-- @return i64?
select max(id) from sql_log;

-- ------------------------------

-- 获取指定ID之后的日志(分机同步用)
-- @name select_after_id
-- @param last_id:i64
-- @return List<SqlLog>
select * from sql_log where id > :last_id order by id limit 100;

-- ------------------------------

-- 获取未执行和执行失败的日志
-- @name select_not_run
-- @return List<SqlLog>
select * from sql_log where state in (0, 2) order by id limit 1000;

-- ------------------------------

-- 设置日志的执行状态
-- @name set_state
-- @param id:i64
-- @param state:i64
-- @param err:String?
update sql_log set state = :state, err = :err where id = :id;

-- ------------------------------

-- 删除某个主机已经同步完成的日志(全量同步之后)
-- @name delete_by_source
-- @param source:String
-- @param id:i64
delete from sql_log where source = :source and id < :id;

-- ------------------------------

-- 删除指定时间之前的日志
-- @name delete_before
-- @param time:i64
delete from sql_log where date < :time;
