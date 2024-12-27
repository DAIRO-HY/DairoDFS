package form

type FilePropertyForm struct{

    /** 名称 **/
    name string

    /** 路径 **/
    path string

    /** 大小 **/
    size string

    /** 文件类型(文件专用) **/
    contentType string

    /** 创建日期 **/
    date string

    /** 是否文件 **/
    isFile bool

    /** 文件数(文件夹属性专用) **/
    fileCount int

    /** 文件夹数(文件夹属性专用) **/
    folderCount int

    /** 历史记录(文件属性专用) **/
    historyList: List<FilePropertyHistoryForm>? = null
}