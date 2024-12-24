package cn.dairo.dfs.controller.app.install.set_storage

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
 * 设置存储目录
 */
@Controller
@RequestMapping("/app/install/set_storage")
class SetStorageInstallAppController : AppBase() {


    @GetMapping
    fun execute() = "app/install/install_set_storage"

    /**
     * 设置存储目录
     */
    @PostMapping("/set")
    @ResponseBody
    fun set(path: Array<String>) {
        SystemConfig.instance.saveFolderList = path.map { it }
        SystemConfig.save()
    }
}
