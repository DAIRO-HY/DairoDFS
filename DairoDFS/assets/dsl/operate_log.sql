-- 按条件分页查询操作日志(按时间倒序)
-- @name select_list
-- @param_type OperateLogQuery
-- @param owner_id:i64? 主账户ID
-- @param user_id:i64? 操作人ID
-- @param op_type:i64? 操作类型
-- @param start:i64? 开始时间
-- @param end:i64? 结束时间
-- @param keyword:String? 路径关键字
-- @param limit:i64
-- @param offset:i64
-- @return List<OperateLog>
select * from operate_log where 1 = 1
-- @if owner_id != null
and ownerId = :owner_id
-- @end
-- @if user_id != null
and userId = :user_id
-- @end
-- @if op_type != null
and opType = :op_type
-- @end
-- @if start != null
and date >= :start
-- @end
-- @if end != null
and date < :end
-- @end
-- @if keyword != null
and path like '%' || :keyword || '%'
-- @end
order by date desc, id desc
limit :limit offset :offset;

-- ------------------------------

-- 按条件统计操作日志数量
-- @name count_list
-- @param_type OperateLogCountQuery
-- @param owner_id:i64? 主账户ID
-- @param user_id:i64? 操作人ID
-- @param op_type:i64? 操作类型
-- @param start:i64? 开始时间
-- @param end:i64? 结束时间
-- @param keyword:String? 路径关键字
-- @return i64
select count(*) from operate_log where 1 = 1
-- @if owner_id != null
and ownerId = :owner_id
-- @end
-- @if user_id != null
and userId = :user_id
-- @end
-- @if op_type != null
and opType = :op_type
-- @end
-- @if start != null
and date >= :start
-- @end
-- @if end != null
and date < :end
-- @end
-- @if keyword != null
and path like '%' || :keyword || '%'
-- @end
;

-- ------------------------------

-- 获取日志里出现过的操作人(用于筛选),owner_id为空时获取全部
-- @name select_users
-- @param owner_id:i64?
-- @return List<OperateLogUser>
select userId, max(userName) as userName from operate_log where 1 = 1
-- @if owner_id != null
and ownerId = :owner_id
-- @end
group by userId order by userId;

-- ------------------------------

-- 删除指定时间之前的日志
-- @name delete_before
-- @param time:i64
delete from operate_log where date < :time;
