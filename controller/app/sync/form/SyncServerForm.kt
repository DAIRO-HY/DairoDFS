package form

type SyncServerForm struct{

    /** 编号 **/
    no int

    /** 主机端同步连接 **/
    url string

    /** 同步状态 0：待机中   1：同步中  2：同步错误 **/
    state int

    /** 同步消息 **/
    msg string

    /** 同步日志数 **/
    syncCount int

    /** 最后一次同步完成时间 **/
    lastTime int64

    /** 最后一次心跳时间 **/
    lastHeartTime int64
}