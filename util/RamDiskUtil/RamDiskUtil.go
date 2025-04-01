package RamDiskUtil

import (
	"DairoDFS/application"
	"os"
)

// RAM硬盘目录
const RAM_DISK_FOLDER = "/mnt/dfs-ramdisk"

// 初始化RAM硬盘
func InitRamDisk() {
	//if runtime.GOOS != "linux" {
	//	return
	//}
	//_, err := ShellUtil.ExecToOkResult("mount -t tmpfs -o size=128M tmpfs /mnt/ramdisk-dfs")
	//if err != nil {
	//}
}

// 获取RAM硬盘目录
func GetRamFolder() string {
	_, err := os.Stat(RAM_DISK_FOLDER)
	if err != nil { //如果又错误
		return application.TEMP_PATH
	}
	return RAM_DISK_FOLDER
}
