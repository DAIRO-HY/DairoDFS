package SystemConfig

import (
	"DairoDFS/extension/String"
	"DairoDFS/util/LogUtil"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"sync"
	"time"
)

/**
 * 系统配置
 */
type SystemConfig struct {

	/**
	 * 记录同步日志
	 */
	OpenSqlLog bool

	/**
	 * 将当前服务器设置为只读,仅作为备份使用
	 */
	IsReadOnly bool

	/**
	 * 文件上传限制(MB)
	 */
	UploadMaxSize uint64

	/**
	 * 文件保存文件夹列表
	 */
	SaveFolderList []string

	/**
	 * 同步域名
	 */
	SyncDomains []string

	/**
	 * 分机与主机同步连接票据
	 */
	Token string
}

// 读取文件锁
var readLock sync.Mutex

// 系统配置的静态实例
var instance *SystemConfig

// 获取系统配置
func Instance() *SystemConfig {
	if instance == nil {
		readLock.Lock()
		_, err := os.Stat(appication.SYSTEM_JSON_PATH)
		if os.IsNotExist(err) { //若配置文件不存在
			timeNow := strconv.FormatInt(time.Now().UnixMilli(), 10)

			//创建一个新的实列
			instance = &SystemConfig{
				UploadMaxSize:  10 * 1024 * 1024 * 1024, //默认文件上传限制10GB
				SaveFolderList: []string{appication.DataPath},
				SyncDomains:    []string{},
				Token:          String.ToMd5(timeNow),
			}
			Save()
		} else {
			instance = &SystemConfig{}
			data, _ := os.ReadFile(appication.SYSTEM_JSON_PATH)
			readJsonErr := json.Unmarshal(data, instance)
			if readJsonErr != nil {
				LogUtil.Error1("读取JSON配置文件失败:", readJsonErr)
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
	_, err := os.Stat(appication.SYSTEM_JSON_PATH)
	if os.IsNotExist(err) { //文件不存在时创建文件夹
		mkdirErr := os.MkdirAll(String.FileParent(appication.SYSTEM_JSON_PATH), os.ModePerm)
		if mkdirErr != nil {
			LogUtil.Error1("文件夹创建失败：", mkdirErr)
			return
		}
	}
	jsonData, _ := json.Marshal(Instance())
	writeErr := os.WriteFile(appication.SYSTEM_JSON_PATH, jsonData, 0644)
	if writeErr != nil {
		fmt.Println(err)
	}
}
