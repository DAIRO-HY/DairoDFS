package File

import (
	"fmt"
	"testing"
)

// 获取文件md5
func TestToMd5(t *testing.T) {
	md5 := ToMd5("C:\\Users\\user\\Desktop\\202411\\dairo-dfs-server-1.0.8.jar")
	fmt.Println(md5)
}
