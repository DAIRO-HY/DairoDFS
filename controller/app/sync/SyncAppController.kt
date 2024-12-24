package cn.dairo.dfs.controller.app.sync

import cn.dairo.dfs.controller.app.sync.form.SyncServerForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.sync.SyncByTable
import cn.dairo.dfs.sync.SyncByLog
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import kotlin.concurrent.thread

/**
 * 数据同步状态
 */
@Controller
@RequestMapping("/app/sync")
class SyncAppController : AppBase() {

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/sync"

    /**
     * 页面数据初始化
     */
    @PostMapping("/info_list")
    @ResponseBody
    fun infoList(): List<SyncServerForm> {
        val formList = SyncByLog.syncInfoList.map {
            val form = SyncServerForm()
            form.url = it.url
            form.state = it.state
            form.msg = it.msg
            form.no = it.no
            form.syncCount = it.syncCount
            form.lastHeartTime = it.lastHeartTime
            form.lastTime = it.lastTime
            form
        }
        return formList
    }

    /**
     * 日志同步
     */
    @PostMapping("/by_log")
    @ResponseBody
    fun sync() {
        thread {
            SyncByLog.start(true)
        }
    }

    /**
     * 全量同步
     */
    @PostMapping("/by_table")
    @ResponseBody
    fun syncAll() {
        thread {
            SyncByTable.start(true)
        }
    }
}
