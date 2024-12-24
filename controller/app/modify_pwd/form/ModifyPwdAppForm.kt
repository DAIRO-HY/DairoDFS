package cn.dairo.dfs.controller.app.modify_pwd.form

import io.swagger.v3.oas.annotations.Parameter
import jakarta.validation.constraints.NotBlank
import org.hibernate.validator.constraints.Length

class ModifyPwdAppForm {

    @Parameter(description = "旧密码")
    @NotBlank
    @Length(min = 4, max = 32)
    var oldPwd: String? = null

    @Parameter(description = "新密码")
    @NotBlank
    @Length(min = 4, max = 32)
    var pwd: String? = null
}