package GoroutineLocal

import (
	"fmt"
	"runtime"
	"strings"
	"sync"
)

// 全局存储
var goroutineLocalStore = sync.Map{}

// 设置 Goroutine 局部存储
func Set(key string, value any) {
	gid := id()
	data, _ := goroutineLocalStore.LoadOrStore(gid, map[string]any{})
	data.(map[string]any)[key] = value
}

// 获取 Goroutine 局部存储
func Get(key string) (any, bool) {
	gid := id()
	data, isExists := goroutineLocalStore.Load(gid)
	if !isExists {
		return nil, false
	}
	value, ok := data.(map[string]any)[key]
	if !ok {
		return nil, false
	}
	return value, true
}

// 移除某个key
func Remove(key string) {
	gid := id()
	data, isExists := goroutineLocalStore.Load(gid)
	if !isExists {
		return
	}
	delete(data.(map[string]any), key)
}

// 移除某个协程的所有数据
func Clear() {
	gid := id()
	goroutineLocalStore.Delete(gid)
}

// 获取 Goroutine ID
func id() string {
	var buf [64]byte
	n := runtime.Stack(buf[:], false)
	return strings.Fields(string(buf[:n]))[1]
}

func Test() {
	var count int
	goroutineLocalStore.Range(func(k any, v any) bool {
		count++
		return true
	})
	fmt.Println(count)
}
