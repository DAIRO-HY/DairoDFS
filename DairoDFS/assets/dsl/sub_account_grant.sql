-- 获取子账户的授权
-- @name select_by_sub_user_id
-- @param sub_user_id:i64
-- @return List<SubAccountGrant>
select * from sub_account_grant where subUserId = :sub_user_id order by id;

-- ------------------------------

-- 删除子账户的授权
-- @name delete_by_sub_user_id
-- @param sub_user_id:i64
delete from sub_account_grant where subUserId = :sub_user_id;
