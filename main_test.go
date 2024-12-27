package main

import (
	"DairoDFS/util/GoroutineLocal"
	"fmt"
	"testing"
	"time"
)

// 协程专有变量测试
func TestGoroutine(t *testing.T) {
	go put()
	time.Sleep(1 * time.Second)
}

func put() {
	GoroutineLocal.Set("key", "123")
	GoroutineLocal.Set("key", "456")
	GoroutineLocal.Set("key", "789")
	get()
}

func get() {
	value, _ := GoroutineLocal.Get("key")
	fmt.Println(value)
}
