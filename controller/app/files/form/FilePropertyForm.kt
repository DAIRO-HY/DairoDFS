package cn.dairo.dfs.controller.app.files.form

class FilePropertyForm {

    /**
     * 名称
     */
    var name: String? = null

    /**
     * 路径
     */
    var path: String? = null

    /**
     * 大小
     */
    var size: String? = null

    /**
     * 文件类型(文件专用)
     */
    var contentType: String? = null

    /**
     * 创建日期
     */
    var date: String? = null

    /**
     * 是否文件
     */
    var isFile: Boolean? = null

    /**
     * 文件数(文件夹属性专用)
     */
    var fileCount: Int? = null

    /**
     * 文件夹数(文件夹属性专用)
     */
    var folderCount: Int? = null

    /**
     * 历史记录(文件属性专用)
     */
    var historyList: List<FilePropertyHistoryForm>? = null
}