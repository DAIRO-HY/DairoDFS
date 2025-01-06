package form

import (
	"DairoDFS/dao/UserDao"
)

type LoginAppInForm struct {

	/** 用户名 **/
	Name *string `validate:"required,min=2,max=32"`

	/** 登录密码(MD5) **/
	Pwd *string `validate:"required,min=2,max=32"`

	/** 设备唯一标识 **/
	DeviceId *string `validate:"required"`
}

// 用户名密码验证
func (mine *LoginAppInForm) IsNameAndPwd() *string {
	msg := "用户名或密码错误"
	userDto := UserDao.SelectByName(*mine.Name)
	if userDto == nil { //用户不存在
		return &msg
	}
	if *mine.Pwd != *userDto.Pwd { //密码不正确
		return &msg
	}
	return nil
}
