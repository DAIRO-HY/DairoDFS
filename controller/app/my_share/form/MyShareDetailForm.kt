package form


type MyShareDetailForm struct{

    /** id **/
    id int64

    /** 链接 **/
    url string

    /** 加密分享 **/
    pwd string

    /** 分享的文件夹 **/
    folder string

    /** 分享的文件夹或文件名,用|分割 **/
    names string

    /** 结束日期 **/
    endDate string

    /** 创建日期 **/
    date string
}