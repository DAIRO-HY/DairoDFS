package cn.dairo.dfs.controller.app.self_set

import cn.dairo.dfs.config.Constant
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.controller.app.self_set.form.SelfSetForm
import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.extension.base64
import cn.dairo.dfs.extension.format
import cn.dairo.dfs.extension.toShortString
import cn.dairo.lib.StringUtil
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody

/**
 * 系统设置
 */
@Controller
@RequestMapping("/app/self_set")
class SelfSetAppController : AppBase() {

    /**
     * 用户操作Dao
     */
    @Autowired
    private lateinit var userDao: UserDao

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/self_set"

    /**
     * 页面初始化
     */
    @PostMapping
    @ResponseBody
    fun init(): SelfSetForm {
        val dto = this.userDao.selectOne(super.loginId)!!
        val form = SelfSetForm()
        form.id = dto.id
        form.name = dto.name
        form.email = dto.email
        form.date = dto.date?.format()
        form.urlPath = dto.urlPath
        form.apiToken = dto.apiToken
        form.encryptionKey = dto.encryptionKey
        return form
    }

    /**
     * 生成API票据
     */
    @PostMapping("/make_api_token")
    @ResponseBody
    fun makeApiToken(flag: Int) {
        val id = super.loginId
        if (flag == 0) {
            this.userDao.setApiToken(id, null)
            return
        }
        val timespan = System.currentTimeMillis() - Constant.BASE_TIME
        val apiToken = StringUtil.getRandomChar(5) + timespan.toShortString
        this.userDao.setApiToken(id, apiToken)
    }

    /**
     * 生成web访问路径前缀
     */
    @PostMapping("/make_url_path")
    @ResponseBody
    fun makeUrlPath(flag: Int) {
        val id = super.loginId
        if (flag == 0) {
            this.userDao.setUrlPath(id, null)
            return
        }
        val timespan = System.currentTimeMillis() - Constant.BASE_TIME
        val urlPath = timespan.toShortString
        this.userDao.setUrlPath(id, urlPath)
    }

    /**
     * 生成端对端加密
     */
    @PostMapping("/make_encryption")
    @ResponseBody
    fun makeEncryption(flag: Int) {
        val id = super.loginId
        if (flag == 0) {
            this.userDao.setEncryptionKey(id, null)
            return
        }
        val encryptionDataArray = ByteArray(128) { it.toByte() }
        encryptionDataArray.shuffle()
        val encryptionKey = encryptionDataArray.base64
        this.userDao.setEncryptionKey(id, encryptionKey)
    }
}
