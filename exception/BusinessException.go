package controller

import "fmt"

// 定义一个自定义错误类型
type BusinessException struct {
	Code    int
	Message string
}

// 实现 error 接口中的 Error() 方法
func (e *BusinessException) Error() string {
	return fmt.Sprintf("Error %d: %s", e.Code, e.Message)
}
