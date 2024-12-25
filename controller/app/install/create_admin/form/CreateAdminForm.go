package form

type CreateAdminForm struct {

	/** 用户名 **/
	Name string `json:"name" validate:"required,min=2,max=32"`

	/** 登录密码 **/
	Pwd string `json:"pwd" validate:"required,min=4,max=32"`
}
