package GoroutineLocal

import (
	"runtime"
	"strings"
	"sync"
)

// 全局存储
var goroutineLocalStore = sync.Map{}

// 设置 Goroutine 局部存储
func Set(key string, value any) {
	gid := GetGoroutineID()
	data, _ := goroutineLocalStore.LoadOrStore(gid, map[string]any{})
	data.(map[string]any)[key] = value
}

// 获取 Goroutine 局部存储
func Get(key string) (any, bool) {
	gid := GetGoroutineID()
	data, isExists := goroutineLocalStore.Load(gid)
	if !isExists {
		return nil, false
	}
	val, ok := data.(map[string]any)[key]
	if !ok {
		return nil, false
	}
	return val, true
}

//移除某个key
func Remove(key string){
	gid := GetGoroutineID()
	data, isExists := goroutineLocalStore.Load(gid)
	if !isExists {
		return
	}
	delete(data.(map[string]any),key)
}

//移除某个协程的所有数据
func RemoveGoroutine(){
	gid := GetGoroutineID()
	goroutineLocalStore.Delete(gid)
}

// 获取 Goroutine ID
func GetGoroutineID() string {
	var buf [64]byte
	n := runtime.Stack(buf[:], false)
	return strings.Fields(string(buf[:n]))[1]
}
