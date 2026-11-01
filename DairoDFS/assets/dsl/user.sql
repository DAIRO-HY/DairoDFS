-- 通过用户名获取用户信息
-- @name select_by_name
-- @param name:String
-- @return User?
select * from user where name = :name;

-- ------------------------------

-- 通过邮箱获取用户信息
-- @name select_by_email
-- @param email:String
-- @return User?
select * from user where email = :email;

-- ------------------------------

-- 获取管理员ID(最早创建的用户就是管理员)
-- @name select_admin_id
-- @return i64?
select id from user order by id limit 1;

-- ------------------------------

-- 通过ApiToken获取用户ID(只查询正常状态的用户)
-- @name select_id_by_api_token
-- @param api_token:String
-- @return i64?
select id from user where apiToken = :api_token and state = 1;

-- ------------------------------

-- 获取用户数量,用来判断系统是否已经初始化
-- @name count
-- @return i64
select count(*) from user;

-- ------------------------------

-- 更新用户信息
-- @name update_info
-- @param id:i64
-- @param name:String
-- @param email:String?
-- @param state:i64
update user set name = :name, email = :email, state = :state where id = :id;

-- ------------------------------

-- 设置URL路径前缀
-- @name set_url_path
-- @param id:i64
-- @param url_path:String?
update user set urlPath = :url_path where id = :id;

-- ------------------------------

-- 设置API票据
-- @name set_api_token
-- @param id:i64
-- @param api_token:String?
update user set apiToken = :api_token where id = :id;

-- ------------------------------

-- 设置端对端加密密钥
-- @name set_encryption_key
-- @param id:i64
-- @param encryption_key:String?
update user set encryptionKey = :encryption_key where id = :id;

-- ------------------------------

-- 设置密码
-- @name set_pwd
-- @param id:i64
-- @param pwd:String
update user set pwd = :pwd where id = :id;

-- ------------------------------

-- 通过WEB访问路径前缀获取用户ID
-- @name select_id_by_url_path
-- @param url_path:String
-- @return i64?
select id from user where urlPath = :url_path and state = 1;
