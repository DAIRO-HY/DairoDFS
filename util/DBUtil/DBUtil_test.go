package DBUtil

import (
	"fmt"
	"testing"
)

func TestID(t *testing.T) {
	var idMap = make(map[int64]bool)
	for i := 0; i < 100; i++ {
		id := ID()
		_, isExits := idMap[id]
		if isExits {
			fmt.Printf("-->%d\n", id)
			t.Error("生成了重复的id")
		}
		idMap[id] = true
	}
}
