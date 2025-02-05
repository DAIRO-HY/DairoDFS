package SyncHandle

import (
	"DairoDFS/dao/dto"
	"encoding/json"
	"fmt"
	"testing"
)

func TestHandle(t *testing.T) {
	result := new(dto.DfsFileDto)
	err := json.Unmarshal([]byte(`{"id":9999999,"name":"文件名"}`), &result)
	fmt.Println(err)
	jsonStr, _ := json.Marshal(result)
	fmt.Println(string(jsonStr))
}
