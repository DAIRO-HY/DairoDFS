package exception

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

// 获取一个异常
func Biz(msg string) *BusinessException {
	return &BusinessException{
		Code:    -1,
		Message: msg,
	}
}

// 获取一个异常
func BizCode(code int, msg string) *BusinessException {
	return &BusinessException{
		Code:    code,
		Message: msg,
	}
}

func FAIL() *BusinessException {
	return &BusinessException{
		Code:    1,
		Message: "操作失败",
	}
}
func EXISTS_NAME() *BusinessException {
	return &BusinessException{
		Code:    1,
		Message: "该用户名已被注册",
	}
}
func EXISTS_EMAIL() *BusinessException {
	return &BusinessException{
		Code:    1,
		Message: "该邮箱已被其他用户注册",
	}
}
func PARAM_ERROR() *BusinessException {
	return &BusinessException{
		Code:    2,
		Message: "参数错误",
	}
}
func SYSTEM_ERROR() *BusinessException {
	return &BusinessException{
		Code:    3,
		Message: "系统错误,请查看错误日志",
	}
}
func SYSTEM_ERROR_NO_LOG() *BusinessException {
	return &BusinessException{
		Code:    3,
		Message: "系统错误,日志未记录",
	}
}
func NOT_ALLOW() *BusinessException {
	return &BusinessException{
		Code:    4,
		Message: "非法操作",
	}
}
func NO_LOGIN() *BusinessException {
	return &BusinessException{
		Code:    5,
		Message: "没有登录",
	}
}
func EXISTS_FILE(name string) *BusinessException {
	return &BusinessException{
		Code:    1001,
		Message: "文件[" + name + "]已存在",
	}
}
func NO_FOLDER() *BusinessException {
	return &BusinessException{
		Code:    1002,
		Message: "文件夹不存在",
	}
}
func EXISTS(name string) *BusinessException {
	return &BusinessException{
		Code:    1003,
		Message: "文件或文件夹[" + name + "]已存在",
	}
}
func NO_EXISTS() *BusinessException {
	return &BusinessException{
		Code:    1004,
		Message: "文件夹或文件不存在",
	}
}
func FILE_UPLOADING() *BusinessException {
	return &BusinessException{
		Code:    1005,
		Message: "文件服务繁忙，请稍后重试。",
	}
}
func SHARE_NOT_FOUND() *BusinessException {
	return &BusinessException{
		Code:    2001,
		Message: "分享链接不存在",
	}
}
func SHARE_IS_END() *BusinessException {
	return &BusinessException{
		Code:    2002,
		Message: "分享已过期",
	}
}
func SHARE_NEED_PWD() *BusinessException {
	return &BusinessException{
		Code:    2003,
		Message: "需要提取码",
	}
}
