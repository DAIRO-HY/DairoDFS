package Sync

import "sync"

// 数据同步锁，防止并发执行
var SyncLock sync.Mutex
