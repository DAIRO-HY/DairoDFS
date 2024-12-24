package cn.dairo.dfs.controller.app.sync.form

class SyncServerForm {

    /**
     * 编号
     */
    var no: Int = 0

    /**
     * 主机端同步连接
     */
    var url: String = ""

    /**
     * 同步状态 0：待机中   1：同步中  2：同步错误
     */
    var state: Int = 0

    /**
     * 同步消息
     */
    var msg: String = ""

    /**
     * 同步日志数
     */
    var syncCount: Int = 0

    /**
     * 最后一次同步完成时间
     */
    var lastTime: Long = 0L

    /**
     * 最后一次心跳时间
     */
    var lastHeartTime: Long = 0L
}