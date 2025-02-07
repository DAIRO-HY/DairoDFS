package app

import (
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/GoroutineLocal"
	"fmt"
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

	// 获取连接池统计信息
	stats := DBUtil.DBConn.Stats()
	fmt.Printf("当前打开的连接数: %d      ", stats.OpenConnections)
	fmt.Printf("正在使用的连接数: %d      ", stats.InUse)
	fmt.Printf("空闲的连接数: %d\n", stats.Idle)
	GoroutineLocal.Test()
	runtime.GC()
}
