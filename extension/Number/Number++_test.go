package Number

import (
	"fmt"
	"sync"
	"testing"
	"time"
)

// 生成ID测试
func TestID(t *testing.T) {
	var idMap = make(map[int64]bool)
	var lock sync.Mutex
	for i := 0; i < 100; i++ {
		go func() {
			id := ID()
			_, isExits := idMap[id]
			if isExits {
				fmt.Printf("-->%d\n", id)
				t.Error("生成了重复的id")
			}
			fmt.Println(id)
			lock.Lock()
			idMap[id] = true
			lock.Unlock()
		}()
	}
	time.Sleep(2 * time.Second)
}
