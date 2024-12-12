package main

import (
	"DairoDFS/appication"
	_ "DairoDFS/util/DBUtil"
	"fmt"
)

func main() {
	appication.Init()
	fmt.Println("Hello, World!")
}
