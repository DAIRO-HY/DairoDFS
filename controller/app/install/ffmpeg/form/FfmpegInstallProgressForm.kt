package cn.dairo.dfs.controller.app.install.ffmpeg.form

class FfmpegInstallProgressForm {


    /**
     * 是否正在下载
     */
    var hasRuning: Boolean = false

    /**
     * 是否已经安装完成
     */
    var hasFinish: Boolean = false

    /**
     * 文件总大小
     */
    var total: String = ""

    /**
     * 已经下载大小
     */
    var downloadedSize: String = ""

    /**
     * 下载速度
     */
    var speed: String = ""

    /**
     * 下载进度
     */
    var progress: Int = 0

    /**
     * 下载url
     */
    var url: String? = null

    /**
     * 安装信息
     */
    var info: String? = null

    /**
     * 错误信息
     */
    var error: String? = null
}