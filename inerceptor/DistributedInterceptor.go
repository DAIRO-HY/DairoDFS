package inerceptor

import (
	"DairoDFS/application/SystemConfig"
	"net/http"
	"strings"
)

// DistributedValidate 分布式同步验证拦截器
// @interceptor:before
// @include:/distributed/**
// @order:2
func DistributedValidate(writer http.ResponseWriter, request *http.Request) bool {
	paths := strings.Split(request.URL.Path, "/")
	if len(paths) < 4 {
		writer.WriteHeader(http.StatusNotFound) // 设置状态码"
		return false
	}

	//得到主机端的同步token
	masterToken := paths[2]

	//得到客户端的同步token
	clentToken := paths[3]
	if masterToken == clentToken {
		// 设置 Content-Type 头部信息
		writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")

		// 设置 HTTP 状态码
		writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
		writer.Write([]byte("禁止本机循环同步"))
		return false
	}
	if masterToken == SystemConfig.Instance().DistributedToken {
		return true
	}
	// 设置 Content-Type 头部信息
	writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")

	// 设置 HTTP 状态码
	writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
	writer.Write([]byte("票据无效"))
	return false
}

//func reject(writer http.ResponseWriter, request *http.Request) {
//	if request.Method == "POST" { //Post请求时
//
//		// 设置 Content-Type 头部信息
//		writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")
//
//		// 设置 HTTP 状态码
//		writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
//		jsonData, _ := json.Marshal(exception.NO_LOGIN())
//		writer.Write(jsonData)
//	} else {
//		http.Redirect(writer, request, "/app/login.html", http.StatusFound)
//		//if (request.getHeader("range") != null) {//可能来自客户端下载
//		//    response.status = 500
//		//} else {
//		//    response.sendRedirect("/app/login")
//		//}
//	}
//}
