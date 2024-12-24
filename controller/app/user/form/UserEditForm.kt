package cn.dairo.dfs.controller.app.user.form

import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.extension.bean
import jakarta.validation.constraints.AssertTrue
import jakarta.validation.constraints.Email
import jakarta.validation.constraints.NotBlank
import org.hibernate.validator.constraints.Length

class UserEditForm {

    /**
     * 主键
     */
    var id: Long? = null

    /**
     * 用户名
     */
    @Length(min = 2, max = 32)
    @NotBlank
    var name: String? = null

    @AssertTrue(message = "用户名已经存在")
    fun isName(): Boolean {
        this.name ?: return true
        val existsUser = UserDao::class.bean.selectByName(this.name!!)
        if (this.id == null) {//创建用户时
            if (existsUser != null) {
                return false
            }
        } else {
            if (existsUser != null && existsUser.id != this.id) {
                return false
            }
        }
        return true
    }

    /**
     * 用户电子邮箱
     */
    @Email
    var email: String? = null

    /**
     * 用户状态
     */
    var state: Int = 1

    /**
     * 创建日期
     */
    var date: String? = null

    /**
     * 密码
     */
    var pwd: String? = null

    @AssertTrue(message = "密码必填")
    fun isPwd(): Boolean {
        if (id == null && pwd.isNullOrBlank()) {//创建用户时密码必填
            return false
        }
        return true
    }
}
