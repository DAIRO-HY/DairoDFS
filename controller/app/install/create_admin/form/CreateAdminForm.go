package form

type CreateAdminForm struct {

	/** 用户名 **/
	Name string `validate:"required,min=2,max=32"`

	/** 登录密码 **/
	Pwd string `validate:"required,min=4,max=32"`
}
