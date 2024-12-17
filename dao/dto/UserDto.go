package dto

import "time"

type UserDto struct {

	/**
	 * 主键
	 */
	Id *int64

	/**
	 * 用户名
	 */
	Name *string

	/**
	 * 登陆密码
	 */
	Pwd *string

	/**
	 * 用户电子邮箱
	 */
	Email *string

	/**
	 * 用户文件访问路径前缀
	 */
	UrlPath *string

	/**
	 * API操作TOKEN
	 */
	ApiToken *string

	/**
	 * 端对端加密密钥
	 */
	EncryptionKey *string

	/**
	 * 用户状态
	 */
	State *int8

	/**
	 * 创建日期
	 */
	Date *time.Time
}
