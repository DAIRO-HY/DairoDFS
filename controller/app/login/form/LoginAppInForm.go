package form

import (
	"DairoDFS/dao/UserDao"
)

type LoginAppInForm struct {

	/** 用户名 **/
	//@NotEmpty
	//@Length(min=2,max=32)
	Name string

	/** 登录密码(MD5) **/
	//@NotEmpty
	//@Length(min=2,max=32)
	Pwd string

	/** 设备唯一标识 **/
	//@NotEmpty
	DeviceId string
}

// 用户名密码验证
func (mine LoginAppInForm) IsNameAndPwd() string {
	msg := "用户名或密码错误"
	userDto, isExists := UserDao.SelectByName(mine.Name)
	if !isExists { //用户不存在
		return msg
	}
	if mine.Pwd != userDto.Pwd { //密码不正确
		return msg
	}
	return ""
}
