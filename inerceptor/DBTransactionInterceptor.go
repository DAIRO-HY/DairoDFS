package inerceptor

import (
	"DairoDFS/util/DBUtil"
	"net/http"
)

// Commit 提交日志
// @interceptor:after
// @include:**
// @order:999999997
func Commit(writer http.ResponseWriter, request *http.Request, body any) any {
	DBUtil.Commit()
	return body
}
