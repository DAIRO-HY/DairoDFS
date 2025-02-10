package form

type SyncServerForm struct {

	/** 编号 **/
	No int `json:"no,omitempty"`

	/** 主机端同步连接 **/
	Url string `json:"url,omitempty"`

	/** 同步状态 0：待机中   1：同步中  2：同步错误 **/
	State int `json:"state,omitempty"`

	/** 同步消息 **/
	Msg string `json:"msg,omitempty"`

	/** 同步日志数 **/
	SyncCount int `json:"syncCount,omitempty"`

	/** 最后一次同步完成时间 **/
	LastTime string `json:"lastTime,omitempty"`

	/** 最后一次心跳时间 **/
	LastHeartTime string `json:"lastHeartTime,omitempty"`
}
