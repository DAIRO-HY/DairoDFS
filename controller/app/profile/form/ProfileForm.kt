package form

import jakarta.validation.constraints.Digits
import jakarta.validation.constraints.Min
import jakarta.validation.constraints.NotBlank

type ProfileForm struct{

    /** 记录同步日志 **/
    openSqlLog bool

    /** 将当前服务器设置为只读,仅作为备份使用 **/
    hasReadOnly bool

    /** 文件上传限制 **/
    @Digits(integer = 11, fraction = 0)
    @NotBlank
    uploadMaxSize string

    /** 存储目录 **/
    @NotBlank
    folders string

    /** 同步域名 **/
    syncDomains string

    /** 分机与主机同步连接票据 **/
    token string
}