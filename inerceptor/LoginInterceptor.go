package inerceptor

import (
	application "DairoDFS/appication"
	"DairoDFS/dao/UserTokenDao"
	"DairoDFS/exception"
	"DairoDFS/util/GoroutineLocal"
	"encoding/json"
	"net/http"
)

// LoginValidate 管理员登录验证
// @interceptor:before
// @include:/**
// @exclude:/app/login**
func LoginValidate(writer http.ResponseWriter, request *http.Request) bool {

	//获取APP登录票据
	cookieToken, _ := request.Cookie("token")
	if cookieToken == nil {
		reject(writer, request)
		return false
	}
	token := cookieToken.Value
	if len(token) == 0 {
		reject(writer, request)
		return false
	}
	userId := UserTokenDao.GetByUserIdByToken(token)
	if userId == nil { //用户未登录
		reject(writer, request)
		return false
	}
	GoroutineLocal.Set(application.USER_ID, userId)
	return true
}

func reject(writer http.ResponseWriter, request *http.Request) {
	if request.Method == "POST" { //Post请求时

		// 设置 Content-Type 头部信息
		writer.Header().Set("Content-Type", "text/plain;charset=UTF-8")

		// 设置 HTTP 状态码
		writer.WriteHeader(http.StatusInternalServerError) // 设置状态码
		jsonData, _ := json.Marshal(exception.NO_LOGIN())
		writer.Write(jsonData)
	} else {
		http.Redirect(writer, request, "/app/login", http.StatusFound)
		//if (request.getHeader("range") != null) {//可能来自客户端下载
		//    response.status = 500
		//} else {
		//    response.sendRedirect("/app/login")
		//}
	}
}
