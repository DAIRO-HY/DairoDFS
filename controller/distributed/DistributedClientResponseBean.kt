package cn.dairo.dfs.controller.distributed

import jakarta.servlet.http.HttpServletResponse

/**
 * 分机端同步response信息
 */
class DistributedClientResponseBean(val clientToken: String, val response: HttpServletResponse) {

    /**
     * 用来标记是否已经取消
     */
    var isCancel: Boolean = false
}