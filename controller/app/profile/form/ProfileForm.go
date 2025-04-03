package form

import (
	"strings"
)

type ProfileForm struct {

	/** 记录同步日志 **/
	OpenSqlLog bool `json:"openSqlLog"`

	/** 将当前服务器设置为只读,仅作为备份使用 **/
	HasReadOnly bool `json:"hasReadOnly"`

	/** 文件上传限制 **/
	//@Digits(integer = 11, fraction = 0)
	//@NotBlank
	UploadMaxSize int64 `json:"uploadMaxSize"`

	/** 存储目录 **/
	//@NotBlank
	Folders string `json:"folders"`

	/** 同步域名 **/
	SyncDomains string `json:"syncDomains"`

	/** 分机与主机同步连接票据 **/
	Token string `json:"token"`

	// 回收站超时(单位：天)
	TrashTimeout int64 `json:"trashTimeout"`

	// 删除没有被使用的文件超时设置(单位：天)
	DeleteStorageTimeout int64 `json:"deleteStorageTimeout"`

	/** 缩略图最大边尺寸 **/
	//@Limit(min = 100, max = 1024)
	ThumbMaxSize int `json:"thumbMaxSize"`

	/** 忽略本机同步错误 **/
	IgnoreSyncError bool `json:"ignoreSyncError"`
}

// 目录正确性检查
func (mine ProfileForm) IsFolders() string {
	if strings.Contains(mine.Folders, "..") {
		return "目录中不能包含点[..]"
	}
	return ""
}
