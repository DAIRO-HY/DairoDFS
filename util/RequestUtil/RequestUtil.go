package RequestUtil

import (
	"net/http"
	"strings"
)

/**
 * 获取客户端ip
 */
func GetIp(request *http.Request) string { //指定读取部分数据头部标识
	var ip string
	ip = request.Header.Get("x-forwarded-for")
	if ip != "" {
		return ip
	}
	ip = request.Header.Get("Proxy-Client-IP")
	if ip != "" {
		return ip
	}
	ip = request.Header.Get("WL-Proxy-Client-IP")
	if ip != "" {
		return ip
	}
	ip = request.Header.Get("HTTP_CLIENT_IP")
	if ip != "" {
		return ip
	}
	ip = request.Header.Get("HTTP_X_FORWARDED_FOR")
	if ip != "" {
		return ip
	}
	ip = request.RemoteAddr
	return ip[:strings.LastIndex(ip, ":")]
}
