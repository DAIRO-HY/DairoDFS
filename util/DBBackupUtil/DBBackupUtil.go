package DBBackupUtil

import (
	"DairoDFS/application"
	"DairoDFS/application/SystemConfig"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBConnection"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/klauspost/compress/flate"
	"github.com/klauspost/compress/zip"
)

// 数据库备份文件名格式
const backupFileDateFoemat = "20060102150405"

// Backup 备份数据库
func Backup() {
	DBConnection.DBConn.Close() //先关闭数据然后备份
	defer DBConnection.Init()   //备份结束之后开启DB链接
	dstZip := application.DB_BACKUP_PATH + "/" + time.Now().Format(backupFileDateFoemat) + ".zip"
	if _, statErr := os.Stat(dstZip); !os.IsNotExist(statErr) { //如果文件已经存在,休眠1秒之后继续
		time.Sleep(1 * time.Second)
		Backup()
		return
	}

	//创建目录
	if err := os.MkdirAll(String.FileParent(dstZip), os.ModePerm); err != nil {
		panic(err)
	}

	// 创建 zip 文件
	zipFile, createErr := os.Create(dstZip)
	if createErr != nil {
		panic(createErr)
	}
	defer zipFile.Close()

	// 创建 zip.Writer
	zipWriter := zip.NewWriter(zipFile)
	defer zipWriter.Close()

	// 注册自定义压缩器（最高压缩比）
	zipWriter.RegisterCompressor(zip.Deflate, func(out io.Writer) (io.WriteCloser, error) {
		return flate.NewWriter(out, flate.BestCompression)
	})

	//将文件写入zip文件
	writeFileToZip(zipWriter, application.SQLITE_PATH)
	writeFileToZip(zipWriter, application.SQLITE_PATH+"-wal")
	writeFileToZip(zipWriter, application.SQLITE_PATH+"-shm")

	//删除过期的备份文件
	deleteExpireFile()
}

// 备份数据库
func writeFileToZip(zipWriter *zip.Writer, path string) {
	if _, err := os.Stat(path); os.IsNotExist(err) { //如果文件不存在
		return
	}

	// 打开源文件
	dbFile, openDBFileErr := os.Open(path)
	if openDBFileErr != nil {
		panic(openDBFileErr)
	}
	defer dbFile.Close()

	// 获取文件信息
	info, statErr := dbFile.Stat()
	if statErr != nil {
		panic(statErr)
	}

	// 创建 zip 条目
	header, fileInfoHeaderErr := zip.FileInfoHeader(info)
	if fileInfoHeaderErr != nil {
		panic(fileInfoHeaderErr)
	}
	header.Name = filepath.Base(path) // 压缩后 zip 内的文件名
	header.Method = zip.Deflate       // 使用 Deflate（会走我们注册的压缩器）

	writer, createHeaderErr := zipWriter.CreateHeader(header)
	if createHeaderErr != nil {
		panic(createHeaderErr)
	}

	// 拷贝数据
	if _, err := io.Copy(writer, dbFile); err != nil {
		panic(err)
	}
}

// 删除过期的备份文件
func deleteExpireFile() {
	entries, err := os.ReadDir(application.DB_BACKUP_PATH)
	if err != nil {
		panic(err)
	}

	//当前时间戳毫秒
	now := time.Now().UnixMilli()
	for _, entry := range entries {
		name := entry.Name()
		if !strings.HasSuffix(name, ".zip") {
			continue
		}

		//得到文件名时间字符串
		datetimeStr := name[:len(name)-4]

		//转换日期时指定时区
		datetime, toDatetimeErr := time.ParseInLocation(backupFileDateFoemat, datetimeStr, time.Local)
		if toDatetimeErr != nil {
			continue
		}
		if (now - datetime.UnixMilli()) > int64(SystemConfig.Instance().DbBackupExpireDay)*24*60*60*1000 {

			//删除文件
			os.Remove(application.DB_BACKUP_PATH + "/" + entry.Name())
		}
	}
}
