package modify_pwd

import (
	"DairoDFS/controller/app/modify_pwd/form"
	"DairoDFS/dao/UserDao"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/extension/String"
	"DairoDFS/util/LoginState"
)

//密码修改
//@Group:/app/modify_pwd

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

// 修改密码
// @Post:/modify
func Modify(inForm form.ModifyPwdAppForm) error {
	userId := LoginState.LoginId()
	newPwd := String.ToMd5(inForm.Pwd)
	UserDao.SetPwd(userId, newPwd)
	UserTokenDao.DeleteByUserId(userId)
	return nil
}
