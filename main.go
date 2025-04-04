package main

import (
	"DairoDFS/application"
	"DairoDFS/controller/app"
	"DairoDFS/util/DistributedUtil/SyncByLog"
	"DairoDFS/util/LogUtil"
	"DairoDFS/util/RecycleStorageTimer"
)

func main() {

	//程序初始化
	application.Init()
	LogUtil.Info("项目启动")

	//启动定时回收器
	RecycleStorageTimer.Init()

	//添加首页路由监听
	app.Home()

	//开启日志同步监听
	SyncByLog.ListenAll()
	startWebServer(application.Args.Port)
}
