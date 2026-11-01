-- 获取某个用户的所有分享(按创建时间倒序)
-- @name select_by_user_id
-- @param user_id:i64
-- @return List<Share>
select * from share where userId = :user_id order by date desc;

-- ------------------------------

-- 删除某个用户的分享
-- @name delete_by_user_id_and_ids
-- @param user_id:i64
-- @param ids:List<i64>
delete from share where userId = :user_id
-- @each ids seq="," open="and id in (" close=")" item="id"
:id
-- @end
;
