package form

type FfmpegInstallProgressForm struct{


    /** 是否正在下载 **/
    hasRuning bool

    /** 是否已经安装完成 **/
    hasFinish bool

    /** 文件总大小 **/
    total string

    /** 已经下载大小 **/
    downloadedSize string

    /** 下载速度 **/
    speed string

    /** 下载进度 **/
    progress int

    /** 下载url **/
    url string

    /** 安装信息 **/
    info string

    /** 错误信息 **/
    error string
}