package distributed

import (
	"DairoDFS/application/SystemConfig"
	"DairoDFS/dao/UserDao"
	"DairoDFS/util/DistributedUtil/SyncByTable"
	"time"
)

/**
 * 分布式部署
 */
//@Group:/app/install/distributed

// @Get:
// @Html:app/install/distributed.html
func Html() {}

/**
 * 设置分布式部署
 */
//@Post:/set
func Set(syncUrl []string) {
	SystemConfig.Instance().SyncDomains = syncUrl
	SystemConfig.Save()

	//全量同步
	SyncByTable.SyncAll()

	for i := 0; i < 5; i++ {
		time.Sleep(1 * time.Second)
		if UserDao.IsInit() { //稍等User表里有数据之后再返回
			return
		}
	}
}
