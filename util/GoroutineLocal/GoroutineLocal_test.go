package GoroutineLocal

import (
	"fmt"
	"testing"
	"time"
)

// 移除某个协程的所有数据
func TestRemoveGoroutine(t *testing.T) {
	RemoveGoroutine()

	Set("time", time.Now().UnixMicro())
	fmt.Println("------------------------------------------")

	// 遍历
	goroutineLocalStore.Range(func(key, value any) bool {
		fmt.Printf("Key: %v, Value: %v\n", key, value)
		return true // 继续遍历
	})

	RemoveGoroutine()
	fmt.Println("------------------------------------------")

	// 遍历
	goroutineLocalStore.Range(func(key, value any) bool {
		fmt.Printf("Key: %v, Value: %v\n", key, value)
		return true // 继续遍历
	})
	fmt.Println("------------------------------------------")
}
