package cn.dairo.dfs.sync

import org.springframework.http.HttpStatus
import java.net.HttpURLConnection
import java.net.URL

/**
 * 同步数据专用的HTTP请求工具类
 */
object SyncHttp {

    /**
     * 请求同步数据
     * @param url 请求url
     * @return 返回结果
     */
    fun request(url: String): String {
        val httpUrl = URL(url)
        val conn = httpUrl.openConnection() as HttpURLConnection
        try {
            conn.requestMethod = "GET"

            //连接超时
            conn.connectTimeout = 3000

            //读数据超时
            conn.readTimeout = 30000
            conn.connect()

            //返回状态码
            val httpStatus = conn.responseCode
            if (httpStatus != HttpStatus.OK.value()) {//请求数据发生错误
                conn.errorStream.use {
                    throw RuntimeException(String(it.readAllBytes()))
                }
            }
            conn.inputStream.use {
                return String(it.readAllBytes())
            }
        } finally {
            conn.disconnect()
        }
    }
}
