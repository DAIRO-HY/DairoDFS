package form

import "time"

type ShareForm struct {

	// 分享结束时间戳,0代表永久有效
	//@NotEmpty
	EndDateTime int64

	// 分享密码
	//@Length(max=32)
	Pwd string

	// 分享的文件夹
	Folder string

	// 要分享的文件名或文件夹名列表
	//@NotEmpty
	Names []string
}

/** 验证截止日期是否正确输入 **/
func (mine ShareForm) IsEndDateTime() string {
	if mine.EndDateTime == 0 {
		return ""
	}
	if mine.EndDateTime < time.Now().UnixMilli() {
		return "结束日期必须在现在的时间之后"
	}
	return ""
}
