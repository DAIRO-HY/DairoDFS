package LoginState

import (
	application "DairoDFS/application"
	"DairoDFS/util/GoroutineLocal"
)

// 获取会员登录id
func LoginId() int64 {
	userId, isExists := GoroutineLocal.Get(application.USER_ID)
	if !isExists {
		return -1
	}
	return userId.(int64)
}
