package form

import (
	"DairoDFS/dao/UserDao"
	"DairoDFS/extension/String"
	"DairoDFS/util/LoginState"
)

type ModifyPwdAppForm struct {

	/** 旧密码 **/
	//@NotBlank
	//@Length(min = 4, max = 32)
	OldPwd string

	/** 新密码 **/
	//@NotBlank
	//@Length(min = 4, max = 32)
	Pwd string
}

// 验证旧密码
func (mine ModifyPwdAppForm) IsOldPwd() string {
	userId := LoginState.LoginId()
	user, _ := UserDao.SelectOne(userId)
	if user.Pwd != String.ToMd5(mine.OldPwd) {
		return "旧密码不正确"
	}
	return ""
}
