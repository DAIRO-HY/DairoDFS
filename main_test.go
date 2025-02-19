package main

import (
	"fmt"
	"testing"
	"time"
)

func TestPanic(t *testing.T) {
	go callPanic()
	time.Sleep(1 * time.Hour)
}

func callPanic() int64 {
	temp := 0
	defer func() {
		fmt.Println(temp)
	}()
	panicFun()
	return 123
}

func panicFun() {
	panic("panic")
}
