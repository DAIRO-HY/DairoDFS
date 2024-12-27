package inerceptor

import (
	"DairoDFS/util/GoroutineLocal"
	"net/http"
)

// 协程局部变量拦截器,每个请求结束之后清除当前协程的局部变量
// @interceptor:after
// @include:/**
// @order:999999999
func RemoveGoroutineLocal(writer http.ResponseWriter, request *http.Request, body any) any {
	GoroutineLocal.RemoveGoroutine()
	return body
}
