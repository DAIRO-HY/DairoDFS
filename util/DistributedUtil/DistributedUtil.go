package DistributedUtil

import (
	"sync"
)

// 数据同步锁，防止并发执行
var SyncLock sync.Mutex

// 标记是否正在全量同步中
var IsTableSyncing = false

// 标记是否正在日志同步中
var IsLogSyncing = false
