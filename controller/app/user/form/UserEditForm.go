package form

import (
	"DairoDFS/dao/UserDao"
)

type UserEditForm struct {

	/** 主键 **/
	Id *int64 `json:"id"`

	/** 用户名 **/
	Name *string `json:"name" validate:"required,min=2,max=32"`

	/** 用户电子邮箱 **/
	Email *string `json:"email" validate:"email"`

	/** 用户状态 **/
	State *int `json:"state"`

	/** 创建日期 **/
	Date *string `json:"date"`

	/** 密码 **/
	Pwd *string `json:"pwd"`
}

func (mine *UserEditForm) IsName() *string {
	msg := "用户名已经存在"
	existsUser := UserDao.SelectByName(*mine.Name)
	if mine.Id == nil { //创建用户时
		if existsUser != nil {
			return &msg
		}
	} else {
		if existsUser != nil && *existsUser.Id != *mine.Id {
			return &msg
		}
	}
	return nil
}

func (mine *UserEditForm) IsPwd() *string {
	msg := "密码必填"
	if mine.Id == nil && mine.Pwd == nil { //创建用户时密码必填
		return &msg
	}
	return nil
}
