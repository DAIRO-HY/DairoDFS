package form

type FileForm struct{

    /** 文件id **/
    id int64

    /** 名称 **/
    name string

    /** 大小 **/
    size int64

    /** 是否文件 **/
    fileFlag bool

    /** 创建日期 **/
    date string

    /** 缩率图 **/
    thumb string
}