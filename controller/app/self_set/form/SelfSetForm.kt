package cn.dairo.dfs.controller.app.self_set.form


class SelfSetForm {


    /**
     * 主键
     */
    var id: Long? = null

    /**
     * 用户名
     */
    var name: String? = null

    /**
     * 用户电子邮箱
     */
    var email: String? = null

    /**
     * 创建日期
     */
    var date: String? = null

    /**
     * 用户文件访问路径前缀
     */
    var urlPath: String? = null

    /**
     * API操作TOKEN
     */
    var apiToken: String? = null

    /**
     * 端对端加密密钥
     */
    var encryptionKey: String? = null
}