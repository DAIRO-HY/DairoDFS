package modify_pwd

import (
	"DairoDFS/controller/app/modify_pwd/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/extension/String"
	"DairoDFS/util/LoginState"
)

/**
 * 密码修改
 */

/**
 * 页面初始化
 */
//@Get:/app/modify_pwd
//@Templates:app/modify_pwd.html
func Html() {}

// 修改密码
// @Post:/app/modify_pwd/modify
func Modify(inForm form.ModifyPwdAppForm) error {
	userId := LoginState.LoginId()
	newPwd := String.ToMd5(inForm.Pwd)
	UserDao.SetPwd(userId, newPwd)
	UserTokenDao.DeleteByUserId(userId)
	return nil
}
