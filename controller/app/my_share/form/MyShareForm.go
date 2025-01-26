package form


type MyShareForm struct{

    /** id **/
    Id int64

    /** 分享的标题（文件名） **/
    Title string

    /** 文件数量 **/
    FileCount int

    /** 是否分享的仅仅是一个文件夹 **/
    FolderFlag bool

    /** 结束时间 **/
    EndDate string

    /** 创建日期 **/
    Date string

    /** 缩略图 **/
    Thumb string
}