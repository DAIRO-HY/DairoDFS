package cn.dairo.dfs.controller.app.profile

import cn.dairo.dfs.config.SystemConfig
import cn.dairo.dfs.controller.app.profile.form.ProfileForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.exception.BusinessException
import cn.dairo.dfs.extension.md5
import cn.dairo.dfs.sync.SyncByLog
import org.springframework.stereotype.Controller
import org.springframework.validation.annotation.Validated
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody
import java.io.File

/**
 * 系统配置
 */
@Controller
@RequestMapping("/app/profile")
class ProfileAppController : AppBase() {

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/profile"

    /**
     * 页面数据初始化
     */
    @PostMapping
    @ResponseBody
    fun init(): ProfileForm {
        val form = ProfileForm()
        val systemConfig = SystemConfig.instance
        form.openSqlLog = systemConfig.openSqlLog
        form.hasReadOnly = systemConfig.isReadOnly
        form.uploadMaxSize = systemConfig.uploadMaxSize.toString()
        form.folders = systemConfig.saveFolderList.joinToString(separator = "\n")
        form.syncDomains = systemConfig.syncDomains.joinToString(separator = "\n")
        form.token = systemConfig.token
        return form
    }

    /**
     * 页面初始化
     */
    @PostMapping("/update")
    @ResponseBody
    fun update(@Validated form: ProfileForm) {
        val folders = form.folders!!.split("\n")
        val systemConfig = SystemConfig.instance
        val saveFolderList = ArrayList<String>()
        folders.forEach {
            if (it.contains(".")) {
                throw BusinessException.addFieldError("folders", "目录中不能包含点[.]")
            }
            val folderFile = File(it)
            if (!folderFile.exists()) {
                throw BusinessException.addFieldError("folders", "目录:${it}不存在")
            }
            saveFolderList.add(folderFile.absolutePath)
        }
        systemConfig.saveFolderList = saveFolderList.distinct()
        systemConfig.uploadMaxSize = form.uploadMaxSize!!.toLong()
        systemConfig.openSqlLog = form.openSqlLog!!
        systemConfig.isReadOnly = form.hasReadOnly!!

        if (form.syncDomains.isNullOrEmpty()) {
            systemConfig.syncDomains = ArrayList()
        } else {

            //配置的同步域名
            val syncDomains = form.syncDomains!!.split("\n").map { it }
            systemConfig.syncDomains = syncDomains
        }
        SyncByLog.init()
        SystemConfig.save()
        SyncByLog.listenAll()
    }

    /**
     * 切换token
     */
    @PostMapping("/make_token")
    @ResponseBody
    fun makeToken() {
        val systemConfig = SystemConfig.instance
        systemConfig.token = System.currentTimeMillis().toString().md5
        SystemConfig.save()
    }
}
