package main

import (
	"DairoDFS/application"
	"DairoDFS/controller/app"
)

func main() {
	application.Init()
	app.Home()
	startWebServer(application.Args.Port)
}
