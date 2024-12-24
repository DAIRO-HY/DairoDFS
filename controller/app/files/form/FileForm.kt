package cn.dairo.dfs.controller.app.files.form

class FileForm {

    /**
     * 文件id
     */
    var id: Long = 0

    /**
     * 名称
     */
    var name: String = ""

    /**
     * 大小
     */
    var size: Long = 0

    /**
     * 是否文件
     */
    var fileFlag: Boolean = false

    /**
     * 创建日期
     */
    var date: String = ""

    /**
     * 缩率图
     */
    var thumb: String? = null
}