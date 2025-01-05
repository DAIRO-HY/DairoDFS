package form

type LoginAppInForm struct {

	/** 用户名 **/
	Name string `validate:"required,min=2,max=32"`

	/** 登录密码(MD5) **/
	Pwd string `validate:"required,min=2,max=32"`

	/** 设备唯一标识 **/
	DeviceId string `validate:"required"`
}

func (mine *LoginAppInForm) IsNameAndPwd() *string {
	msg := "这是一段测试错误"
	return &msg
}
