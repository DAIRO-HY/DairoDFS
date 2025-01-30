package app

import (
	"net/http"
	"runtime"
)

// 页面初始化
func Home() {

	//首页跳转
	http.HandleFunc("/", func(writer http.ResponseWriter, request *http.Request) {
		path := request.URL.Path
		if path != "/" { //未知的请求连接
			writer.WriteHeader(http.StatusNotFound) // 设置状态码
			writer.Write([]byte("404 page not found"))
			return
		}
		if request.Method == "GET" {
			http.Redirect(writer, request, "/index.html", http.StatusFound)
		}
	})
}

// 页面初始化
// @Html:index.html
func Index() {
	runtime.GC()
}
