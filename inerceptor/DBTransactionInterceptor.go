package inerceptor

import (
	"DairoDFS/util/DBConnection"
	"net/http"
)

// Commit 提交日志
// @interceptor:after
// @include:**
// @order:999999997
func Commit(writer http.ResponseWriter, request *http.Request, body any) any {
	if _, ok := body.(error); ok { //程序发生了错误
		DBConnection.Rollback()
	} else {
		DBConnection.Commit()
	}
	return body
}
