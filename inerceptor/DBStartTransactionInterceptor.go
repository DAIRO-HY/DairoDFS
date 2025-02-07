package inerceptor

import (
	"DairoDFS/util/DBConnection"
	"net/http"
)

// Commit 提交日志
// @interceptor:before
// @include:**
// @order:0
func StartTransaction(writer http.ResponseWriter, request *http.Request) bool {
	DBConnection.SetAutoCommit(false)
	return true
}
