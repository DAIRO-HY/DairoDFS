-- 添加一条日志,顺序号在同一条语句里取当前最大值加1,同时添加日志时也不会重复
-- @name insert_with_seq
-- @param id:i64
-- @param log_time:i64
-- @param sql:String
-- @param param:String
-- @param state:i64
-- @param source:String
-- @param err:String?
insert into sql_log(id, date, sql, param, state, source, err, seq)
values (:id, :log_time, :sql, :param, :state, :source, :err, (select coalesce(max(seq), 0) + 1 from sql_log));

-- ------------------------------

-- 获取当前日志中最大的顺序号
-- @name select_last_seq
-- @return i64?
select max(seq) from sql_log;

-- ------------------------------

-- 获取指定顺序号之后的日志(分机同步用)
-- @name select_after_seq
-- @param last_seq:i64
-- @return List<SqlLog>
select * from sql_log where seq > :last_seq order by seq limit 100;

-- ------------------------------

-- 获取未执行和执行失败的日志(按添加到本机的顺序执行)
-- @name select_not_run
-- @return List<SqlLog>
select * from sql_log where state in (0, 2) order by seq limit 1000;

-- ------------------------------

-- 设置日志的执行状态
-- @name set_state
-- @param id:i64
-- @param state:i64
-- @param err:String?
update sql_log set state = :state, err = :err where id = :id;

-- ------------------------------

-- 删除某个主机还没有执行的日志(全量同步之后,这些修改已经包含在全量同步的数据里)
-- @name delete_not_run_by_source
-- @param source:String
delete from sql_log where source = :source and state != 1;

-- ------------------------------

-- 删除指定时间之前的日志
-- @name delete_before
-- @param time:i64
delete from sql_log where date < :time;
