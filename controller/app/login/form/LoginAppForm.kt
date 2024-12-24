package cn.dairo.dfs.controller.app.login.form

import io.swagger.v3.oas.annotations.Parameter
import jakarta.validation.constraints.NotBlank
import org.hibernate.validator.constraints.Length

class LoginAppForm {

    @Parameter(description = "用户名")
    @Length(min = 2, max = 32)
    @NotBlank
    var name: String? = null

    @Parameter(description = "登录密码(MD5)")
    @NotBlank
    var pwd: String? = null

    @Parameter(description = "设备唯一标识")
    @NotBlank
    var deviceId: String? = null
}