package SystemConfig

import (
	"DairoDFS/application"
	"DairoDFS/extension/String"
	"DairoDFS/util/LogUtil"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"sync"
)

// 系统配置
type SystemConfig struct {

	// 记录同步日志
	OpenSqlLog bool

	// 将当前服务器设置为只读,仅作为备份使用
	IsReadOnly bool

	// 文件上传限制(MB)
	UploadMaxSize int64

	// 文件保存文件夹列表
	SaveFolderList []string

	// 同步域名
	SyncDomains []string

	// 分机与主机同步连接票据
	DistributedToken string

	// 回收站超时(单位：天)
	TrashTimeout int64

	// 删除没有被使用的文件超时设置(单位：天)
	DeleteStorageTimeout int64

	// 缩略图最大边尺寸
	ThumbMaxSize int

	// 忽略本机同步错误
	IgnoreSyncError bool

	// 数据库备份天数
	DbBackupExpireDay int
}

// 读取文件锁
var readLock sync.Mutex

// 系统配置的静态实例
var instance *SystemConfig

// 获取系统配置
func Instance() *SystemConfig {
	if instance == nil {
		readLock.Lock()
		_, err := os.Stat(application.SYSTEM_JSON_PATH)
		if os.IsNotExist(err) { //若配置文件不存在

			//创建一个新的实列
			instance = &SystemConfig{
				UploadMaxSize:        10 * 1024 * 1024 * 1024, //默认文件上传限制10GB
				SaveFolderList:       []string{application.DataPath},
				SyncDomains:          []string{},
				DistributedToken:     String.MakeRandStr(32),
				TrashTimeout:         30,
				DeleteStorageTimeout: 30,
				ThumbMaxSize:         360,
				DbBackupExpireDay:    30,
			}
			Save()
		} else {
			instance = &SystemConfig{}
			data, _ := os.ReadFile(application.SYSTEM_JSON_PATH)
			readJsonErr := json.Unmarshal(data, instance)
			if readJsonErr != nil {
				LogUtil.Error3("读取JSON配置文件失败:", readJsonErr)
				log.Fatal(readJsonErr)
			}
		}
		readLock.Unlock()
	}
	return instance
}

/**
 * 数据持久化
 */
func Save() {
	_, err := os.Stat(application.SYSTEM_JSON_PATH)
	if os.IsNotExist(err) { //文件不存在时创建文件夹
		mkdirErr := os.MkdirAll(String.FileParent(application.SYSTEM_JSON_PATH), os.ModePerm)
		if mkdirErr != nil {
			LogUtil.Error3("文件夹创建失败：", mkdirErr)
			return
		}
	}
	jsonData, _ := json.Marshal(Instance())
	writeErr := os.WriteFile(application.SYSTEM_JSON_PATH, jsonData, 0644)
	if writeErr != nil {
		fmt.Println(err)
	}
}
