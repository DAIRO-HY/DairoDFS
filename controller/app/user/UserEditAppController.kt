package cn.dairo.dfs.controller.app.user

import cn.dairo.dfs.code.ErrorCode
import cn.dairo.dfs.controller.app.user.form.UserEditForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.dao.dto.UserDto
import cn.dairo.dfs.extension.format
import cn.dairo.dfs.service.UserService
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.validation.annotation.Validated
import org.springframework.web.bind.annotation.*

/**
 * 用户编辑
 */
@Controller
@RequestMapping("/app/user_edit")
class UserEditAppController : AppBase() {
    companion object {
        const val PWD_PLACEHOLDER = "********************************"
    }

    /**
     * 用户操作Dao
     */
    @Autowired
    private lateinit var userDao: UserDao

    /**
     * 用户操作Service
     */
    @Autowired
    private lateinit var userService: UserService

    /**
     * 初始化
     */
    @GetMapping
    fun execute() = "app/user_edit"

    /**
     * 页面初始化
     */
    @PostMapping
    @ResponseBody
    fun init(id: Long?): UserEditForm {
        val form = UserEditForm()
        if (id != null) {
            val dto = this.userDao.selectOne(id)!!
            form.id = dto.id
            form.name = dto.name
            form.pwd = PWD_PLACEHOLDER
            form.email = dto.email
            form.date = dto.date?.format()
            form.state = dto.state!!
        }
        return form
    }

    /**
     * 添加或更新数据
     */
    @PostMapping("/edit")
    @ResponseBody
    fun edit(@Validated form: UserEditForm) {
        val dto = UserDto()
        dto.id = form.id
        dto.name = form.name
        dto.email = form.email
        dto.state = form.state
        try {
            if (form.id == null) {
                this.userService.add(dto)
            } else {
                this.userDao.update(dto)
            }
            if (form.pwd != PWD_PLACEHOLDER) {//更新密码
                this.userDao.setPwd(dto.id!!, form.pwd!!)
            }
        } catch (e: Exception) {
            val message = e.message ?: throw e
            if (message.contains("UNIQUE constraint failed: user.email")) {//该邮箱已被其他用户注册
                throw ErrorCode.EXISTS_EMAIL
            }
            throw e
        }
    }
}
