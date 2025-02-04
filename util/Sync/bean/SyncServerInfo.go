package bean

type SyncServerInfo struct {

	/**
	 * 编号
	 */
	No int

	/**
	 * 主机端同步连接
	 */
	Url string

	/**
	 * 同步状态 0：待机中   1：同步中  2：同步错误
	 */
	State int

	/**
	 * 同步消息
	 */
	Msg string

	/**
	 * 同步日志数
	 */
	SyncCount int

	/**
	 * 最后一次同步完成时间
	 */
	LastTime int64

	/**
	 * 最后一次心跳时间
	 */
	LastHeartTime int64
}
