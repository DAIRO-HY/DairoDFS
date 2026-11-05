-- 获取子账户的授权
-- @name select_by_sub_user_id
-- @param sub_user_id:i64
-- @return List<SubAccountGrant>
select * from sub_account_grant where subUserId = :sub_user_id order by id;

-- ------------------------------

-- 修改授权的权限
-- @name set_perms
-- @param id:i64
-- @param perms:i64
update sub_account_grant set perms = :perms where id = :id;
