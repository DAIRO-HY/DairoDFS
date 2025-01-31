package cn.dairo.dfs.sync.bean

import java.net.HttpURLConnection

/**
 * 同步日志请求对象Bean
 */
class SyncLogListenHttpBean(val http: HttpURLConnection) {

    /**
     * 是否取消
     */
    var isCanceled: Boolean = false
    fun cancle() {
        this.isCanceled = true
        this.http.disconnect()
    }
}