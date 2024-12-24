package cn.dairo.dfs.controller.app.my_share.form


class MyShareDetailForm {

    /**
     * id
     */
    var id: Long? = null

    /**
     * 链接
     */
    var url: String? = null

    /**
     * 加密分享
     */
    var pwd: String? = null

    /**
     * 分享的文件夹
     */
    var folder: String? = null

    /**
     * 分享的文件夹或文件名,用|分割
     */
    var names: String? = null

    /**
     * 结束日期
     */
    var endDate: String? = null

    /**
     * 创建日期
     */
    var date: String? = null
}