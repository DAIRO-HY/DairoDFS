package form

import io.swagger.v3.oas.annotations.Parameter
import jakarta.validation.constraints.NotBlank
import org.hibernate.validator.constraints.Length

type ModifyPwdAppForm struct{

    @Parameter(description = "旧密码")
    @NotBlank
    @Length(min = 4, max = 32)
    oldPwd string

    @Parameter(description = "新密码")
    @NotBlank
    @Length(min = 4, max = 32)
    pwd string
}