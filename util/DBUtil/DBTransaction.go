package DBUtil

import (
	"DairoDFS/util/DBSqlLog"
	"DairoDFS/util/GoroutineLocal"
	"database/sql"
	_ "github.com/mattn/go-sqlite3"
)

// 事务数据库协程本地变量
const _TRANSACTION_KEY = "DB"

// 获取数据库连接
func getConnection() *sql.DB {
	return DBConn
}

// 获取数据库连接
//func getConnection() *sql.Tx {
//	tx, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
//	if isExists {
//		return tx.(*sql.Tx)
//	}
//	createTx, err := DBConn.Begin()
//	if err != nil {
//		//@TODO:
//	}
//	GoroutineLocal.Set(_TRANSACTION_KEY, createTx)
//	return createTx
//}

// 提交事务
func Commit() {
	//value, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
	//if isExists {
	//	tx := value.(*sql.Tx)
	//	DBSqlLog.SaveLog(tx)
	//	tx.Commit()
	//	DBSqlLog.Clear()
	//	GoroutineLocal.Remove(_TRANSACTION_KEY)
	//}
}

// 回滚事务
func Rollback() {
	tx, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
	if isExists {
		tx.(*sql.Tx).Rollback()
		DBSqlLog.Clear()
		GoroutineLocal.Remove(_TRANSACTION_KEY)
	}
}
