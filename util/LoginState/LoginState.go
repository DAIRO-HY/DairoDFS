package LoginState

import (
	application "DairoDFS/application"
	"DairoDFS/dao/UserDao"
	"DairoDFS/util/GoroutineLocal"
)

// 记录当前管理员id
var adminId int64

// 判断当前登录的用户是否管理员
func IsAdmin() bool {
	if adminId == 0 {
		adminId = UserDao.SelectAdminId()
	}
	return adminId == LoginId()
}

// 获取会员登录id
func LoginId() int64 {
	userId, isExists := GoroutineLocal.Get(application.USER_ID)
	if !isExists {
		return -1
	}
	return userId.(int64)
}
