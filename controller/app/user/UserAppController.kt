package cn.dairo.dfs.controller.app.user

import cn.dairo.dfs.controller.app.user.form.UserInfoForm
import cn.dairo.dfs.controller.base.AppBase
import cn.dairo.dfs.dao.UserDao
import io.swagger.v3.oas.annotations.Operation
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.stereotype.Controller
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.ResponseBody

/**
 * 用户API
 */
@Controller
@RequestMapping("/app/user")
class UserAppController : AppBase() {

    /**
     * 用户操作Dao
     */
    @Autowired
    private lateinit var userDao: UserDao

    @Operation(summary = "获取用户信息")
    @PostMapping("/get_user_info")
    @ResponseBody
    fun getUserInfo(): UserInfoForm {
        val dto = this.userDao.selectOne(super.loginId)!!
        val form = UserInfoForm()
        form.id = dto.id
        form.name = dto.name
//        form.adminFlag = super.isAdmin
        return form
    }
}
