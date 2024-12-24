package form

type CreateAdminForm struct {

	/** 用户名 **/
	//@Length(min = 2, max = 32)
	//@NotBlank
	Name string

	/** 登录密码 **/
	//@Length(min = 4, max = 32)
	//@NotBlank
	Pwd string
}
