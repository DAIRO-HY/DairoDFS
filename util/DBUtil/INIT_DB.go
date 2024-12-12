package DBUtil

import (
	"DairoDFS/resources"
	"DairoDFS/util/LogUtil"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strconv"
)

// VERSION 数据库版本号
const VERSION = 2

func init() {
	_, err := os.Stat(DB_PATH)
	// 如果错误是os.ErrNotExist，表示文件不存在
	if os.IsNotExist(err) { //文件不存在

		// 创建多层目录
		err := os.MkdirAll(filepath.Dir(DB_PATH), 0700)
		if err != nil {
			LogUtil.Error(fmt.Sprintf("创建文件夹[%s]失败 err:%q", DB_PATH, err))
			log.Fatal(err)
			return
		}
	}

	// 打开数据库连接，没有文件时会自动创建
	//db, _ := sql.Open("sqlite3", DB_PATH)
	upgrade()
}

/**
* 更新表结构
 */
func upgrade() {
	version := SelectSingleOneIgnoreError[int]("PRAGMA USER_VERSION")
	if version == 0 {
		create()

	}
	if version > 0 {
	}

	//设置数据库版本号
	ExecIgnoreError("PRAGMA USER_VERSION = " + strconv.Itoa(VERSION))
}

func create() {
	sqlFiles := []string{"dfs_file.sql", "local_file.sql", "share.sql", "share_file.sql", "sql_log.sql", "user.sql", "user_token.sql"}
	for _, fn := range sqlFiles {
		createSql, _ := resources.SqlFolder.ReadFile("sql/create/" + fn)
		ExecIgnoreError(string(createSql))
	}
}
