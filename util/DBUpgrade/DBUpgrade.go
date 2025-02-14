package DBUpgrade

import (
	"DairoDFS/resources"
	"database/sql"
	"strconv"
	"strings"
)

// VERSION 数据库版本号
const _VERSION = 3

/**
* 更新表结构
 */
func Upgrade(db *sql.DB) {
	var version int
	db.QueryRow("PRAGMA USER_VERSION").Scan(&version)
	if version == 0 {
		create(db)
	}
	if version > 0 {
	}

	//设置数据库版本号
	db.Exec("PRAGMA USER_VERSION = " + strconv.Itoa(_VERSION))
}

func create(db *sql.DB) {
	sqlFiles := []string{"dfs_file.sql", "storage_file.sql", "share.sql", "share_file.sql", "sql_log.sql", "user.sql", "user_token.sql"}
	for _, fn := range sqlFiles {
		createSql, _ := resources.SqlFolder.ReadFile("sql/create/" + fn)
		db.Exec(string(createSql))
	}

	//将dfs_file表复制一份,用来保存彻底删除的数据
	dfsFileDeleteData, _ := resources.SqlFolder.ReadFile("sql/create/dfs_file.sql")
	dfsFileDeleteSql := string(dfsFileDeleteData)
	dfsFileDeleteSql = strings.ReplaceAll(dfsFileDeleteSql, "dfs_file", "dfs_file_delete")
	db.Exec(dfsFileDeleteSql)
}
