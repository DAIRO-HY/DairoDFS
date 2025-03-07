package main

import (
	"DairoDFS/extension/Number"
	"DairoDFS/extension/String"
	"DairoDFS/util/DBConnection"
	"DairoDFS/util/DBUtil"
	"DairoDFS/util/LogUtil"
	"database/sql"
	"fmt"
	"log"
	"strconv"
	"strings"
	"testing"
	"time"
)

// 旧数据迁移
func TestUpdateOldDb(t *testing.T) {
	start := time.Now()
	DBConnection.StartTransaction()
	allData := getAllData()
	addToNewDB(allData)
	//DBConnection.Rollback()
	DBConnection.Commit()
	fmt.Println(time.Since(start))
}

func addToNewDB(dataMap map[string][]map[string]any) {
	for _, it := range dataMap["user"] {
		date := it["date"].(time.Time)
		sql := "insert into user(id,name,pwd,email,urlPath,apiToken,encryptionKey,state,date)values (?,?,?,?,?,?,?,?,?)"
		_, err := DBUtil.Insert(sql, cutId(it["id"]), it["name"], it["pwd"], it["email"], it["urlPath"], it["apiToken"], it["encryptionKey"], it["state"], date.UnixMilli())
		if err != nil {
			panic(err)
		}
	}

	storageOldIdToNewIdMap := make(map[int64]int64)
	for _, it := range dataMap["local_file"] {
		newId := Number.ID()
		storageOldIdToNewIdMap[it["id"].(int64)] = newId
		sql := "insert into storage_file(id,path,md5)values (?,?,?)"
		_, err := DBUtil.Insert(sql, newId, it["path"], it["md5"])
		if err != nil {
			fmt.Println(it)
			fmt.Println(err)
			panic(err)
		}
	}

	fileOldIdToNewIdMap := make(map[int64]int64)
	for _, it := range dataMap["dfs_file"] {
		newId := Number.ID()
		fileOldIdToNewIdMap[it["id"].(int64)] = newId

		date := it["date"].(time.Time)
		name := it["name"].(string)
		storageId := it["localId"].(int64)
		if storageId != 0 {
			oldStorageId := storageId
			storageId = storageOldIdToNewIdMap[storageId]
			if storageId == 0 {
				fmt.Println(oldStorageId)
			}
		}
		sql := "insert into dfs_file(id,userId,parentId,name,ext,size,contentType,storageId,date,property,isExtra,isHistory,deleteDate,state,stateMsg)values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"
		_, err := DBUtil.Insert(sql,
			newId,
			cutId(it["userId"]),
			it["parentId"],
			name,
			strings.ToLower(String.FileExt(name)),
			it["size"],
			it["contentType"],
			storageId,
			date.UnixMilli(),
			it["property"],
			it["isExtra"],
			it["isHistory"],
			it["deleteDate"],
			it["state"],
			it["stateMsg"],
		)
		if err != nil {
			fmt.Println(it)
			fmt.Println(err)
			panic(err)
		}
	}
	for oldId, newId := range fileOldIdToNewIdMap {
		DBUtil.Exec("update dfs_file set parentId = ? where parentId = ?", newId, oldId)
	}
}

func cutId(idData any) int64 {
	idStr := strconv.FormatInt(idData.(int64), 10)
	idStr = idStr[:len(idStr)-3]
	id, _ := strconv.ParseInt(idStr, 10, 64)
	return id
}

func getAllData() map[string][]map[string]any {
	dataMap := make(map[string][]map[string]any)
	for _, it := range []string{"dfs_file", "local_file", "user"} {
		dataList := selectToListMap("select * from " + it)
		dataMap[it] = dataList
	}
	return dataMap
}

func getDB() *sql.DB {

	//_busy_timeout：设置数据库被锁时超时
	//经过验证，在journal_mode=WAL模式下，如果开启事务，只有遇到了修改语句才会锁库，select语句不会锁库。
	//数据库被锁之后其他任何线程，包括其他程序都无法对数据库操作。
	//数据库被锁之后，查询语句不影响
	db, err := sql.Open("sqlite3", "dairo-dfs.sqlite")
	if err != nil {
		LogUtil.Error(fmt.Sprintf("打开数据库失败 err:%q", err))
		log.Fatal(err)
	}
	return db
}

// SelectToListMap 将查询结果以List<Map>的类型返回
func selectToListMap(query string, args ...any) []map[string]any {
	rows, err := getDB().Query(query, args...)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("查询数据失败:%s: err:%q", query, err))
		return nil
	}
	defer rows.Close()

	// 获取列的名称
	columns, err := rows.Columns()
	if err != nil {
		LogUtil.Error(fmt.Sprintf("%q: %s\n", err, query))
		return nil
	}

	// 创建一个[]interface{}的slice, 每个元素指向values中的对应位置
	valuePtrs := make([]any, len(columns))

	// 创建一个空切片
	list := make([]map[string]any, 0) // 初始化空切片
	for rows.Next() {

		// 创建一个长度与列数相同的slice来存放查询结果
		values := make([]any, len(columns))
		for i := range values {
			valuePtrs[i] = &values[i]
		}

		// 将当前行的数据扫描到valuePtrs中
		if err := rows.Scan(valuePtrs...); err != nil {
			LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, err))
			return nil
		}

		// 使用map将列名和对应的值关联起来
		rowMap := make(map[string]any)
		for i, col := range columns {
			value := values[i]
			rowMap[col] = value
		}
		list = append(list, rowMap)
	}
	return list
}
