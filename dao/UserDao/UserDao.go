package UserDao

import (
	"DairoDFS/dao/dto"
	"DairoDFS/util/DBUtil"
)

/**
 * 添加一条数据
 * @param dto 用户信息
 */
func Add(dto dto.UserDto) {
	DBUtil.InsertIgnoreError("insert into user(id, name, pwd, email, encryptionKey, state, date) values (?, ?, ?, ?, ?, ?, ?)",
		dto.Id, dto.Name, dto.Pwd, dto.Email, dto.EncryptionKey, dto.State, dto.Date)
}

/**
 * 通过id获取一条数据
 * @param id 用户ID
 * @return 用户信息
 */
func SelectOne(id int64) (dto.UserDto, bool) {
	return DBUtil.SelectOne[dto.UserDto]("select * from user where id = ?", id)
}

/**
 * 获取管理员账户
 * @return 用户信息
 */
func SelectAdminId() int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64]("select id from user order by id limit 1")
}

/**
 * 通过邮箱获取用户信息
 * @param email 邮箱
 * @return 用户信息
 */
func SelectByEmail(email string) (dto.UserDto, bool) {
	return DBUtil.SelectOne[dto.UserDto]("select * from user where email = ?", email)
}

/**
 * 通过用户名获取用户信息
 * @param name 用户名
 * @return 用户信息
 */
func SelectByName(name string) (dto.UserDto, bool) {
	return DBUtil.SelectOne[dto.UserDto]("select * from user where name = ?", name)
}

/**
 * 通过ApiToken获取用户ID
 * @param apiToken 用户ApiToken
 * @return 用户ID
 */
func SelectIdByApiToken(apiToken string) int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64](`select id from user where apiToken = ? and state = 1`, apiToken)
}

/**
 * 通过urlPath获取用户ID
 * @param urlPath 文件访问前缀
 * @return 用户ID
 */
func SelectIdByUrlPath(urlPath string) int64 {
	return DBUtil.SelectSingleOneIgnoreError[int64]("select id from user where urlPath = ? and state = 1", urlPath)
}

/**
 * 获取所有用户
 * @return 所有用户列表
 */
func SelectAll() []dto.UserDto {
	return DBUtil.SelectList[dto.UserDto]("select * from user")
}

/**
 * 判断是否已经初始化
 */
func IsInit() bool {
	return DBUtil.SelectSingleOneIgnoreError[bool]("select count(*) > 0 from user")
}

/**
 * 更新用户信息
 * @param dto 用户信息
 */
func Update(dto dto.UserDto) {
	DBUtil.ExecIgnoreError("update user set name  = ?, email = ?, state = ? where id = ?",
		dto.Name, dto.Email, dto.State, dto.Id)
}

/**
 * 设置URL路径前缀
 * @param id 用户ID
 * @param urlPath URL路径前缀
 */
func SetUrlPath(id int64, urlPath *string) {
	DBUtil.ExecIgnoreError("update user set urlPath = ? where id = ?", urlPath, id)
}

/**
 * 设置API票据
 * @param id 用户ID
 * @param apiToken URL路径前缀
 */
func SetApiToken(id int64, apiToken *string) {
	DBUtil.ExecIgnoreError("update user set apiToken = ? where id = ?", apiToken, id)
}

/**
 * 设置端对端加密
 * @param id 用户ID
 * @param encryptionKey URL路径前缀
 */
func SetEncryptionKey(id int64, encryptionKey *string) {
	DBUtil.ExecIgnoreError(`update user set encryptionKey = ? where id = ?`, encryptionKey, id)
}

/**
 * 设置密码
 * @param id 用户ID
 * @param pwd 密码
 */
func SetPwd(id int64, pwd string) {
	DBUtil.ExecIgnoreError(`update user set pwd = ? where id = ?`, pwd, id)
}
