package form

import cn.dairo.dfs.dao.UserDao
import cn.dairo.dfs.extension.bean
import jakarta.validation.constraints.AssertTrue
import jakarta.validation.constraints.Email
import jakarta.validation.constraints.NotBlank
import org.hibernate.validator.constraints.Length

type UserEditForm struct{

    /** 主键 **/
    id int64

    /** 用户名 **/
    @Length(min = 2, max = 32)
    @NotBlank
    name string

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

    /** 用户电子邮箱 **/
    @Email
    email string

    /** 用户状态 **/
    state int

    /** 创建日期 **/
    date string

    /** 密码 **/
    pwd string

    @AssertTrue(message = "密码必填")
    fun isPwd(): Boolean {
        if (id == null && pwd.isNullOrBlank()) {//创建用户时密码必填
            return false
        }
        return true
    }
}
