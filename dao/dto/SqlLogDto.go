package dto

/**
 * sql数据库日志
 */
type SqlLogDto struct {

	/**
	 * 主键
	 */
	Id int64

	/**
	 * 日志时间
	 */
	Date int64

	/**
	 * sql文
	 */
	Sql string

	/**
	 * 参数Json
	 */
	Param string

	/**
	 * 状态 0：待执行 1：执行完成 2：执行失败
	 */
	State int

	/**
	 * 日志来源IP
	 */
	Source string

	/**
	 * 错误消息
	 */
	Err string
}
