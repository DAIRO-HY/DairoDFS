package inerceptor

import (
	"DairoDFS/util/LoginState"
	"net/http"
)

// AdminValidate 是否管理员验证
// @interceptor:before
// @include:/app/advanced**,/app/profile**,/app/sync**,/app/user**
// @order:2
func AdminValidate(writer http.ResponseWriter, request *http.Request) bool {
	if LoginState.IsAdmin() {
		return true
	}
	writer.WriteHeader(http.StatusUnauthorized) // 设置状态码
	writer.Write([]byte("权限不足"))
	return false
}
