package DistributedPush

import (
	"sync"
	"time"
)

// 长连接心跳间隔时间(秒)
const KEEP_ALIVE_TIME = 120

// 等待信号
var Cond *sync.Cond

func init() {
	var lock sync.Mutex
	Cond = sync.NewCond(&lock)
	go func() {
		for {
			time.Sleep(KEEP_ALIVE_TIME * time.Second)

			//通知释放锁信号
			Cond.Broadcast()
		}
	}()
}

// Push 通知分机端同步
func Push() {
	Cond.Broadcast()
}
