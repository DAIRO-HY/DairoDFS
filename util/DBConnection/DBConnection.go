package DBConnection

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"DairoDFS/exception"
	"DairoDFS/util/DBSqlLog"
	"DairoDFS/util/DBUpgrade"
	"DairoDFS/util/GoroutineLocal"
	"DairoDFS/util/LogUtil"
	"database/sql"
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	"log"
	"strings"
	"time"
)

// 事务数据库协程本地变量
const _START_KEY = "START_KEY"

// 事务数据库协程本地变量
const _TRANSACTION_KEY = "DB"

// 事务数据库协程本地变量
const _AUTO_COMMIT_KEY = "AUTO_COMMIT_KEY"

// sqlite数据库连接对象
var DBConn *sql.DB

func init() {

	//_busy_timeout：设置数据库被锁时超时
	//经过验证，在journal_mode=WAL模式下，如果开启事务，只有遇到了修改语句才会锁库，select语句不会锁库。
	//数据库被锁之后其他任何线程，包括其他程序都无法对数据库操作。
	//数据库被锁之后，查询语句不影响
	db, err := sql.Open("sqlite3", application.SQLITE_PATH+"?_busy_timeout=10000")
	if err != nil {
		LogUtil.Error(fmt.Sprintf("打开数据库失败 err:%q", err))
		log.Fatal(err)
	}

	db.SetMaxOpenConns(20)               // 设置最大打开连接数，值越大支持的并发越高，但常驻内存也会增加。默认无限，大并发可能会导致内存激增。建议设置
	db.SetMaxIdleConns(3)                // 设置最大空闲连接数
	db.SetConnMaxLifetime(1 * time.Hour) // SQLite 通常是文件数据库，不需要 SetConnMaxLifetime，默认让连接长期存活。

	//设置数据库被锁时超时，由于通过sql.Open打开的是一个数据库连接池，所以这里设置可能不生效，推介通过连接参数设置
	//db.Exec("PRAGMA busy_timeout = 100000;")
	//if err1 != nil {
	//	fmt.Println(err1)
	//}

	//DELETE:（默认）
	//适用于大多数单线程或低并发应用。
	//每次事务提交后，SQLite 会删除 journal 文件。
	//事务可靠性高，文件不会增长。
	//适用场景：桌面应用、小型嵌入式系统等。

	//TRUNCATE:
	//和 DELETE 类似，但提交事务时不会删除 journal 文件，而是清空它。
	//优点：避免频繁创建和删除 journal 文件，稍微提高性能。
	//适用场景：低并发但写入频繁的应用（比如定期记录日志）。
	//其他模式（了解）

	//WAL:（适用于高并发）：适合多个读者、较少写者的场景（如 Web 服务器）。
	//MEMORY：事务日志存放在内存中，速度最快，且较少磁盘IO，但掉电数据可能丢失，但使用commit之后数据不会丢失。
	//PERSIST：保留 journal 文件，但不清空内容，适合避免文件创建开销。
	//OFF：完全关闭事务日志，不推荐（容易数据损坏）。
	db.Exec("PRAGMA journal_mode=WAL;")
	DBUpgrade.Upgrade(db)
	DBConn = db
}

// 获取数据库连接
func getConnection() any {
	if _, isExists := GoroutineLocal.Get(_START_KEY); !isExists { //没有开启事务的情况
		return DBConn
	}
	tx, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
	if isExists {
		return tx.(*sql.Tx)
	}
	createTx, err := DBConn.Begin()
	if err != nil {
		//@TODO:
	}
	GoroutineLocal.Set(_TRANSACTION_KEY, createTx)
	return createTx
}

// 设置提交事务方式
// flag ture：自动提交 false：手动提交
func SetAutoCommit(flag bool) {
	_, isExists := GoroutineLocal.Get(_AUTO_COMMIT_KEY)
	if flag {
		if isExists {
			GoroutineLocal.Remove(_AUTO_COMMIT_KEY)
		}
	} else {
		if !isExists {
			GoroutineLocal.Set(_AUTO_COMMIT_KEY, struct{}{})
		}
	}
}

// 是否自动提交事务
func IsAutoCommit() bool {
	_, isExists := GoroutineLocal.Get(_AUTO_COMMIT_KEY)
	return !isExists
}

// 开始事务
func StartTransaction() {
	_, isExists := GoroutineLocal.Get(_START_KEY)
	if isExists {
		return
	}
	GoroutineLocal.Set(_START_KEY, struct{}{})
}

// 提交事务
func Commit() {
	value, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
	if isExists {
		tx := value.(*sql.Tx)
		tx.Commit()
		DBSqlLog.Insert(DBConn)
		DBSqlLog.Clear()
		GoroutineLocal.Remove(_TRANSACTION_KEY)
	}
	GoroutineLocal.Remove(_START_KEY)
}

// 回滚事务
func Rollback() {
	tx, isExists := GoroutineLocal.Get(_TRANSACTION_KEY)
	if isExists {
		tx.(*sql.Tx).Rollback()
		DBSqlLog.Clear()
		GoroutineLocal.Remove(_TRANSACTION_KEY)
	}
	GoroutineLocal.Remove(_START_KEY)
}

// 执行写数据操作
func Write(query string, args ...any) (sql.Result, error) {
	systemConfig := SystemConfig.Instance()
	if systemConfig.IsReadOnly {
		if !strings.Contains(query, "user_token") { // 过滤掉对user_token的操作，保证还能正常登录
			panic(exception.Biz("当前设置为只读模式，该操作不允许"))
		}
	}
	isAutoCommit := IsAutoCommit()
	if !isAutoCommit { //手动提交表单的话，开启事务
		StartTransaction()
	}
	switch value := getConnection().(type) {
	case *sql.DB:
		r, e := value.Exec(query, args...)
		if systemConfig.OpenSqlLog && e == nil { //没有发生错误的情况下，保存执行的sql
			DBSqlLog.Add(query, args)
			if isAutoCommit { //自动提交事务时，需要手动将数据保存到DB
				DBSqlLog.Insert(DBConn)
			}
		}
		return r, e
	case *sql.Tx:
		r, e := value.Exec(query, args...)
		if systemConfig.OpenSqlLog && e == nil { //没有发生错误的情况下，保存执行的sql
			DBSqlLog.Add(query, args)
			if isAutoCommit { //自动提交事务时，需要手动将数据保存到DB
				DBSqlLog.Insert(DBConn)
			}
		}
		return r, e
	default:
		return nil, nil
	}
}

// 执行查询数据操作
func QueryRow(query string, args ...any) *sql.Row {
	switch value := getConnection().(type) {
	case *sql.DB:
		return value.QueryRow(query, args...)
	case *sql.Tx:
		return value.QueryRow(query, args...)
	default:
		return nil
	}
}

// 执行查询数据操作
func Query(query string, args ...any) (*sql.Rows, error) {
	switch value := getConnection().(type) {
	case *sql.DB:
		return value.Query(query, args...)
	case *sql.Tx:
		return value.Query(query, args...)
	default:
		return nil, nil
	}
}
