package form

import (
	"DairoDFS/dao/UserDao"
)

type UserEditInoutForm struct {

	/** 主键 **/
	Id int64 `json:"id"`

	/** 用户名 **/
	//@NotEmpty
	//@Length(min=2,max=32)
	Name string `json:"name"`

	/** 用户电子邮箱 **/
	//@Email
	Email string `json:"email"`

	/** 用户状态 **/
	State int8 `json:"state"`

	/** 创建日期 **/
	Date string `json:"date"`

	/** 密码 **/
	Pwd string `json:"pwd"`
}

// 验证用户名
func (mine UserEditInoutForm) IsName() string {
	msg := "用户名已经存在"
	user, isExists := UserDao.SelectByName(mine.Name)
	if mine.Id == 0 { //创建用户时
		if isExists {
			return msg
		}
	} else {
		if isExists && user.Id != mine.Id {
			return msg
		}
	}
	return ""
}

// 验证密码
func (mine UserEditInoutForm) IsPwd() string {
	msg := "密码必填"
	if mine.Id == 0 && mine.Pwd == "" { //创建用户时密码必填
		return msg
	}
	return ""
}

// 邮箱验证
func (mine UserEditInoutForm) IsEmail() string {
	msg := "该邮箱已经被其他用户使用"
	user, isExists := UserDao.SelectByEmail(mine.Email)
	if isExists && user.Id != mine.Id {
		return msg
	}
	return ""
}
