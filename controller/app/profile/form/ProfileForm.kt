package cn.dairo.dfs.controller.app.profile.form

import jakarta.validation.constraints.Digits
import jakarta.validation.constraints.Min
import jakarta.validation.constraints.NotBlank

class ProfileForm {

    /**
     * 记录同步日志
     */
    var openSqlLog: Boolean? = null

    /**
     * 将当前服务器设置为只读,仅作为备份使用
     */
    var hasReadOnly: Boolean? = null

    /**
     * 文件上传限制
     */
    @Digits(integer = 11, fraction = 0)
    @NotBlank
    var uploadMaxSize: String? = null

    /**
     * 存储目录
     */
    @NotBlank
    var folders: String? = null

    /**
     * 同步域名
     */
    var syncDomains: String? = null

    /**
     * 分机与主机同步连接票据
     */
    var token: String? = null
}