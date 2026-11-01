-- 通过登录Token获取用户ID
-- @name select_user_id_by_token
-- @param token:String
-- @return i64?
select userId from user_token where token = :token;

-- ------------------------------

-- 获取某个用户的登录记录(按登录时间升序)
-- @name select_by_user_id
-- @param user_id:i64
-- @return List<UserToken>
select * from user_token where userId = :user_id order by date;

-- ------------------------------

-- 删除某个用户在某台设备上的登录记录
-- @name delete_by_user_id_and_device_id
-- @param user_id:i64
-- @param device_id:String
delete from user_token where userId = :user_id and deviceId = :device_id;

-- ------------------------------

-- 删除某个用户的所有登录记录
-- @name delete_by_user_id
-- @param user_id:i64
delete from user_token where userId = :user_id;

-- ------------------------------

-- 通过token删除登录记录
-- @name delete_by_token
-- @param token:String
delete from user_token where token = :token;
