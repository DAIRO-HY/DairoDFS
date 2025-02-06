package GoroutineLocal

import (
	"fmt"
	"testing"
	"time"
)

// 移除某个协程的所有数据
func TestRemoveGoroutine(t *testing.T) {
	Clear()

	Set("time", time.Now().UnixMicro())
	fmt.Println("------------------------------------------")

	// 遍历
	goroutineLocalStore.Range(func(key, value any) bool {
		fmt.Printf("Key: %v, Value: %v\n", key, value)
		return true // 继续遍历
	})

	Clear()
	fmt.Println("------------------------------------------")

	// 遍历
	goroutineLocalStore.Range(func(key, value any) bool {
		fmt.Printf("Key: %v, Value: %v\n", key, value)
		return true // 继续遍历
	})
	fmt.Println("------------------------------------------")
}

func TestGetGoroutineID(t *testing.T) {
	count := 1000000
	now := time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		sdf := id()
		if sdf == "sdf" {
			fmt.Printf("OK")
		}
	}
	times := time.Now().UnixMilli() - now
	fmt.Printf("timr-point-总 = %d毫秒\n", times)
	fmt.Printf("timr-point-均 = %.10f毫秒\n\n", float64(times)/float64(count))
}
