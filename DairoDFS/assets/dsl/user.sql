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

-- 获取管理员ID(最早创建的主账户就是管理员)
-- @name select_admin_id
-- @return i64?
select id from user where parentId = 0 order by id limit 1;

-- ------------------------------

-- 获取所有主账户(用户管理)
-- @name select_main_list
-- @return List<User>
select * from user where parentId = 0 order by id;

-- ------------------------------

-- 获取主账户的子账户(不包括已删除的)
-- @name select_sub_list
-- @param parent_id:i64
-- @return List<User>
select * from user where parentId = :parent_id and state != 2 order by id;

-- ------------------------------

-- 获取主账户的所有子账户(包括已删除的)
-- @name select_sub_list_all
-- @param parent_id:i64
-- @return List<User>
select * from user where parentId = :parent_id order by id;

-- ------------------------------

-- 更新子账户信息
-- @name update_sub
-- @param id:i64
-- @param name:String
-- @param nickname:String?
-- @param state:i64
update user set name = :name, nickname = :nickname, state = :state where id = :id;

-- ------------------------------

-- 删除子账户:保留记录,文件和日志里还能显示昵称;清空用户名,之后可以重新使用这个用户名
-- 写成参数的形式:同步时同一条记录在其他服务器上有更新的修改,需要保留其中一部分字段的值
-- @name set_sub_deleted
-- @param id:i64
-- @param state:i64 固定为2
-- @param name:String? 固定为空
-- @param pwd:String? 固定为空
-- @param api_token:String? 固定为空
-- @param url_path:String? 固定为空
update user set state = :state, name = :name, pwd = :pwd, apiToken = :api_token, urlPath = :url_path where id = :id;

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

-- ------------------------------

-- 设置昵称
-- @name set_nickname
-- @param id:i64
-- @param nickname:String?
update user set nickname = :nickname where id = :id;

-- ------------------------------

-- 设置头像
-- @name set_avatar
-- @param id:i64
-- @param avatar:String?
update user set avatar = :avatar where id = :id;

-- ------------------------------

-- 设置是否处理上传的文件
-- @name set_handle_file
-- @param id:i64
-- @param handle_file:i64
update user set handleFile = :handle_file where id = :id;
