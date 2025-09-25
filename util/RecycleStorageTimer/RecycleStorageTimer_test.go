package RecycleStorageTimer

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"testing"
)

func init() {
	application.SQLITE_PATH = "C:\\Users\\ths.developer.1\\IdeaProjects\\DairoDFS\\data\\dairo-dfs.sqlite"
}

func TestStart(t *testing.T) {
	sc := SystemConfig.Instance()
	//sc.DeleteStorageTimeout = 0
	sc.TrashTimeout = 0
	start()
}
