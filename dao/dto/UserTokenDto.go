package dto

type UserTokenDto struct {

	/**
	 * 主键
	 */
	Id int64

	/**
	 * 登录Token
	 */
	Token string

	/**
	 * 用户ID
	 */
	UserId int64

	/**
	 * 客户端标识  0:WEB 1：Android  2：IOS  3：WINDOWS 4:MAC 5:LINUX
	 */
	ClientFlag int

	/**
	 * 设备唯一标识
	 */
	DeviceId string

	/**
	 * 客户端IP地址
	 */
	Ip string

	/**
	 * 创建日期
	 */
	Date int64

	/**
	 * 客户端版本
	 */
	Version int
}
