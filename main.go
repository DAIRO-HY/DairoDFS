package main

import (
	"DairoDFS/application"
	"DairoDFS/controller/app"
)

func main() {
	application.Init()
	app.Home()
	startWebServer(8031)

	//testMap := make(map[int64]bool)
	//testMap[1] = true
	//go func() {
	//	for {
	//		//testMap[time.Now().UnixMilli()] = true
	//		fmt.Println(testMap[1])
	//	}
	//}()
	//for {
	//	//testMap[time.Now().UnixMilli()] = true
	//	fmt.Println(testMap[1])
	//}
}
