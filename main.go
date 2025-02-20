package main

import (
	"DairoDFS/application"
	"DairoDFS/controller/app"
	"DairoDFS/util/SyncUtil/SyncByLog"
)

func main() {
	application.Init()
	app.Home()

	//开启日志同步监听
	SyncByLog.ListenAll()
	startWebServer(application.Args.Port)
}
