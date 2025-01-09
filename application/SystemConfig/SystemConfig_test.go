package SystemConfig

import (
	"fmt"
	"testing"
)

// 获取系统配置
func TestInstance(t *testing.T) {
	config := Instance()
	config = Instance()
	fmt.Println(config)
}
