package cn.dairo.dfs.controller.app.my_share.form


class MyShareForm {

    /**
     * id
     */
    var id: Long? = null

    /**
     * 分享的标题（文件名）
     */
    var title: String? = null

    /**
     * 文件数量
     */
    var fileCount: Int? = null

    /**
     * 是否分享的仅仅是一个文件夹
     */
    var folderFlag: Boolean? = null

    /**
     * 结束时间
     */
    var endDate: String? = null

    /**
     * 创建日期
     */
    var date: String? = null

    /**
     * 缩略图
     */
    var thumb: String? = null
}