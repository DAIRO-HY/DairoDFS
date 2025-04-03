package DistributedUtil

import (
	"DairoDFS/util/DBConnection"
	"context"
	"database/sql"
)

type SyncServerInfo struct {

	// 编号
	No int

	// 主机端同步连接
	Url string

	// 同步状态 0：待机中   1：同步中  2：同步错误
	State int

	// 同步消息
	Msg string

	// 要同步的数据总量
	Count int

	// 同步日志数
	SyncCount int

	// 最后一次同步完成时间
	LastTime int64

	// 最后一次心跳时间
	LastHeartTime int64

	//标记是否已经停止
	IsStop bool

	// 取消函数
	CancelFunc context.CancelFunc

	//数据库事务
	tx *sql.Tx
}

// 取消正在同步的任务
func (mine *SyncServerInfo) Cancel() {
	if mine.CancelFunc != nil {
		mine.CancelFunc()
	}
	mine.IsStop = true
}

// 获取数据库操作事务
func (mine *SyncServerInfo) DbTx() *sql.Tx {
	if mine.tx == nil { //没有事务则开启一个新的事务
		mine.tx, _ = DBConnection.DBConn.Begin()
	}
	return mine.tx
}

// 提交事务
func (mine *SyncServerInfo) Commit() error {
	if mine.tx == nil { //没有事务则开启一个新的事务
		return nil
	}
	err := mine.tx.Commit()
	mine.tx = nil
	return err
}

// 回滚事务
func (mine *SyncServerInfo) Rollback() {
	if mine.tx != nil {
		mine.tx.Rollback()
		mine.tx = nil
	}
}
