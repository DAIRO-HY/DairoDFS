package DBUpgrade

import (
	"DairoDFS/resources"
	"database/sql"
	"strconv"
	"strings"
)

// VERSION 数据库版本号
const _VERSION = 7

/**
* 更新表结构
 */
func Upgrade(db *sql.DB) {
	var version int
	db.QueryRow("PRAGMA USER_VERSION").Scan(&version)
	if version == 0 {
		create(db)
	} else if version < 4 {

		//删除附属文件
		db.Exec("delete from dfs_file where isExtra = 1")

		//将所有文件标记为未处理
		db.Exec("update dfs_file set state = 0 where 1 = 1")
	} else if version < 5 {

		//添加索引
		db.Exec("CREATE INDEX idx_dfs_file_storageId ON dfs_file (storageId);")
	} else if version < 7 {
		db.Exec("drop index idx_dfs_file_storageId;")
		db.Exec("drop index idx_ext;")
		db.Exec("drop index idx_isExtra;")
		db.Exec("drop index idx_userId;")
		db.Exec("drop index index_state;")
	}

	//设置数据库版本号
	db.Exec("PRAGMA USER_VERSION = " + strconv.Itoa(_VERSION))
}

func create(db *sql.DB) {
	sqlFiles := []string{"dfs_file.sql", "storage_file.sql", "share.sql", "sql_log.sql", "user.sql", "user_token.sql"}
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
