package cn.dairo.dfs.controller.app.modify_pwd

import cn.dairo.dfs.controller.app.modify_pwd.form.ModifyPwdAppForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.dao.UserTokenDao
import cn.dairo.dfs.exception.BusinessException
import io.swagger.v3.oas.annotations.Operation
import jakarta.validation.Valid
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody

/**
 * 密码修改
 */
@Controller
@RequestMapping("/app/modify_pwd")
class ModifyAppController : AppBase() {

    /**
     * 用户操作Dao
     */
    @Autowired
    private lateinit var userDao: UserDao

    /**
     * 用户登录票据操作Dao
     */
    @Autowired
    private lateinit var userTokenDao: UserTokenDao

    /**
     * 页面初始化
     */
    @GetMapping
    fun execute() = "app/modify_pwd"

    @Operation(summary = "修改密码")
    @PostMapping("/modify")
    @ResponseBody
    fun modify(@Valid form: ModifyPwdAppForm) {
        val userId = super.loginId
        val user = this.userDao.selectOne(userId)!!
        if (user.pwd != form.oldPwd) {
            throw BusinessException("旧密码不正确")
        }
        this.userDao.setPwd(userId, form.pwd!!)
        this.userTokenDao.deleteByUserId(userId);
    }
}
