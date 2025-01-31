package inerceptor

import (
	"net/http"
	"time"
)

// HtmlInterceptor Html页面拦截器
// @interceptor:after
// @include:**.html
// @order:999999998
func HtmlInterceptor(writer http.ResponseWriter, request *http.Request, body any) any {

	// 设置Cache-Control头，配置缓存（1年）
	writer.Header().Set("Cache-Control", "public, max-age=31536000, s-maxage=31536000, immutable")

	// 设置Expires头，配置为1年后的时间
	expiresTime := time.Now().AddDate(1, 0, 0).Format(time.RFC1123)
	writer.Header().Set("Expires", expiresTime)
	return body
}
