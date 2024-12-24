package cn.dairo.dfs.controller.app.install.distributed

import cn.dairo.dfs.config.SystemConfig
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.sync.SyncByLog
import cn.dairo.dfs.sync.SyncByTable
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import kotlin.concurrent.thread

/**
 * 分布式部署
 */
@Controller
@RequestMapping("/app/install/distributed")
class DistributedInstallAppController : AppBase() {


    @GetMapping
    fun execute() = "app/install/install_distributed"

    /**
     * 设置分布式部署
     */
    @PostMapping("/set")
    @ResponseBody
    fun set(syncUrl: Array<String>) {
        SystemConfig.instance.syncDomains = syncUrl.map { it }
        SystemConfig.save()
        SyncByLog.init()
        thread {

            //全量同步
            SyncByTable.start(true)

            //开启日志监听
            SyncByLog.listenAll()
        }
    }
}
