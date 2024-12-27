package form


type MyShareForm struct{

    /** id **/
    id int64

    /** 分享的标题（文件名） **/
    title string

    /** 文件数量 **/
    fileCount int

    /** 是否分享的仅仅是一个文件夹 **/
    folderFlag bool

    /** 结束时间 **/
    endDate string

    /** 创建日期 **/
    date string

    /** 缩略图 **/
    thumb string
}