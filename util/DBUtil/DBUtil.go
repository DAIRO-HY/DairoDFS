package DBUtil

import (
	"DairoDFS/util/CommonUtil"
	"DairoDFS/util/DBSqlLog"
	"DairoDFS/util/LogUtil"
	"database/sql"
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	"log"
	"reflect"
	"strings"
	"time"
)

// DB_PATH 文件路径
const DB_PATH = "./data/dairo-dfs.sqlite"

// sqlite数据库连接对象
var DBConn *sql.DB

func init() {
	//_busy_timeout：设置数据库被锁时超时
	db, err := sql.Open("sqlite3", DB_PATH+"?_busy_timeout=10000")
	if err != nil {
		LogUtil.Error(fmt.Sprintf("打开数据库失败 err:%q", err))
		log.Fatal(err)
	}

	db.SetMaxOpenConns(10)               // 设置最大打开连接数，值越大支持的并发越高，但常驻内存也会增加。默认无限，大并发可能会导致内存激增。建议设置
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
	DBConn = db
}

// 执行sql语句,忽略错误
func ExecIgnoreError(query string, args ...any) int64 {
	count, err := Exec(query, args...)
	if err != nil {
		log.Printf("%q: %s\n", err, query)
		return -1
	}
	return count
}

// 执行sql
func Exec(query string, args ...any) (int64, error) {
	rs, err := getConnection().Exec(query, args...)
	if err != nil {
		return -1, err
	}
	count, err := rs.RowsAffected()
	if err != nil {
		return -1, err
	}
	DBSqlLog.Add(query, args)
	return count, nil
}

// 添加数据,忽略错误
func InsertIgnoreError(query string, args ...any) int64 {
	count, err := Insert(query, args...)
	if err != nil {
		LogUtil.Error(fmt.Sprintf("添加数据失败:%s  err:%q\n", query, err))
		return -1
	}
	return count
}

// 添加数据,并返回最后一次添加的ID
func Insert(query string, args ...any) (int64, error) {
	rs, err := getConnection().Exec(query, args...)
	if err != nil {
		return -1, err
	}
	lastInsertId, err := rs.LastInsertId()
	if err != nil {
		return -1, err
	}
	DBSqlLog.Add(query, args)
	return lastInsertId, nil
}

// SelectSingleOneIgnoreError 查询第一个数据并忽略错误
func SelectSingleOneIgnoreError[T any](query string, args ...any) T {
	value, _ := SelectSingleOne[T](query, args...)
	return value
}

// SelectSingleOne 查询第一个数据
func SelectSingleOne[T any](query string, args ...any) (T, error) {
	row := getConnection().QueryRow(query, args...)
	var value *T

	// 使用 Scan 将结果赋值给 value
	// 这里最好使用指针的指针类型，否则可能导致string类型为nil时报错
	err := row.Scan(&value)
	if err != nil {
		LogUtil.Debug(fmt.Sprintf("error: %q, sql: %s", err, query))
		return *new(T), err // 返回默认值和错误
	}
	return *value, nil
}

// SelectOne 查询第一个数据
func SelectOne[T any](query string, args ...any) (T, bool) {
	dtoList := SelectList[T](query, args...)
	if len(dtoList) == 0 {
		return *new(T), false
	}
	return dtoList[0], true
}

// SelectList 查询列表
func SelectListBk[T any](query string, args ...any) []*T {
	list := SelectToListMap(query, args...)

	// 创建一个空切片
	dtoList := make([]*T, 0) // 初始化空切片
	for _, item := range list {
		dtoT := new(T)
		reflectDto := reflect.ValueOf(dtoT).Elem()
		for key := range item { //遍历查询到的数据
			value := item[key]
			if value == nil { //该值为空
				continue
			}

			//结构体中的变量名
			varName := strings.ToUpper(string(key[0])) + key[1:]
			field := reflectDto.FieldByName(varName)
			if !field.IsValid() { //结构体中不存在该变量
				continue
			}
			setValue(field, value)
		}
		dtoList = append(dtoList, dtoT)
	}
	return dtoList
}

// SelectList 查询列表
func SelectList[T any](query string, args ...any) []T {
	rows, err := getConnection().Query(query, args...)
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

	//用来存储扫描值的列表
	scanList := make([]any, len(columns))
	for i := 0; i < len(columns); i++ {
		scanList[i] = new(any) //为了解决null值数据，使用双重指针
	}

	// 存储实体类里的指针
	fieldPointList := make([]any, len(columns))

	// 创建一个空切片
	dtoList := make([]T, 0) // 初始化空切片
	for rows.Next() {
		dtoT := new(T)
		if CommonUtil.IsBaseType(*dtoT) { //这是一个基本数据类型
			if scanErr := rows.Scan(dtoT); scanErr != nil {
				LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
				return nil
			}
			dtoList = append(dtoList, *dtoT)
			continue
		}
		reflectDto := reflect.ValueOf(dtoT).Elem()
		for i, column := range columns {

			//结构体中的变量名
			varName := strings.ToUpper(string(column[0])) + column[1:]
			field := reflectDto.FieldByName(varName)
			if field.IsValid() { //获取指针，忽略结构体中不存在该变量
				fieldPointList[i] = field.Addr().Interface()
			}
		}

		// 将当前行的数据扫描到scanList中
		if scanErr := rows.Scan(scanList...); scanErr != nil {
			LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
			return nil
		}
		for i, it := range fieldPointList {
			value := *scanList[i].(*any)
			if value == nil {
				continue
			}
			switch fieldPoint := it.(type) {
			case *int:
				*fieldPoint = int(value.(int64))
			case *int8:
				*fieldPoint = int8(value.(int64))
			case *int16:
				*fieldPoint = int16(value.(int64))
			case *int32:
				*fieldPoint = int32(value.(int64))
			case *int64:
				*fieldPoint = value.(int64)
			case *float64:
				*fieldPoint = value.(float64)
			case *float32:
				*fieldPoint = float32(value.(float64))
			case *bool:
				switch typeValue := value.(type) {
				case int:
					*fieldPoint = typeValue != 0
				case int8:
					*fieldPoint = typeValue != 0
				case int16:
					*fieldPoint = typeValue != 0
				case int32:
					*fieldPoint = typeValue != 0
				case int64:
					*fieldPoint = typeValue != 0
				default:
					*fieldPoint = value.(bool)
				}
			case *string:
				*fieldPoint = value.(string)
			case *time.Time:
				*fieldPoint = value.(time.Time)
			case **int:
				temp := int(value.(int64))
				*fieldPoint = &temp
			case **int8:
				temp := int8(value.(int64))
				*fieldPoint = &temp
			case **int16:
				temp := int16(value.(int64))
				*fieldPoint = &temp
			case **int32:
				temp := int32(value.(int64))
				*fieldPoint = &temp
			case **int64:
				temp := value.(int64)
				*fieldPoint = &temp
			case **float64:
				temp := value.(float64)
				*fieldPoint = &temp
			case **float32:
				temp := float32(value.(float64))
				*fieldPoint = &temp
			case **bool:
				temp := value.(bool)
				*fieldPoint = &temp
			case **string:
				temp := value.(string)
				*fieldPoint = &temp
			case **time.Time:
				temp := value.(time.Time)
				*fieldPoint = &temp
			}
		}
		dtoList = append(dtoList, *dtoT)
	}
	return dtoList
}

// SelectListNull 查询列表
// 该函数是备用，实际测试结果性能比SelectList差一点点
// 测试件数：1000000

// SelectList-总时间 = 1359毫秒
// SelectList-平均时间 = 0.0013590000毫秒
// SelectListNull-总时间 = 1841毫秒
// SelectListNull-平均时间 = 0.0018410000毫秒
func SelectListNull[T any](query string, args ...any) []T {
	rows, err := getConnection().Query(query, args...)
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

	//用来存储扫描值的列表
	scanList := make([]any, len(columns))
	for i := 0; i < len(columns); i++ {
		scanList[i] = new(any) //为了解决null值数据，使用双重指针
	}

	// 存储实体类里的指针
	fieldPointList := make([]any, len(columns))

	// 创建一个空切片
	dtoList := make([]T, 0) // 初始化空切片
	for rows.Next() {
		dtoT := new(T)
		if CommonUtil.IsBaseType(*dtoT) { //这是一个基本数据类型
			if scanErr := rows.Scan(dtoT); scanErr != nil {
				LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
				return nil
			}
			dtoList = append(dtoList, *dtoT)
			continue
		}
		reflectDto := reflect.ValueOf(dtoT).Elem()
		for i, column := range columns {

			//结构体中的变量名
			varName := strings.ToUpper(string(column[0])) + column[1:]
			field := reflectDto.FieldByName(varName)
			if !field.IsValid() { //获取指针，忽略结构体中不存在该变量
				continue
			}
			fieldAddr := field.Addr().Interface()
			switch fieldAddr.(type) { //为不是指针类型的数据做特殊处理，否则数据库中的null无法扫描到dto中
			case *int:
				scanList[i] = new(sql.NullInt64)
				fieldPointList[i] = fieldAddr
			case *int8:
				scanList[i] = new(sql.NullInt16)
				fieldPointList[i] = fieldAddr
			case *int16:
				scanList[i] = new(sql.NullInt16)
				fieldPointList[i] = fieldAddr
			case *int32:
				scanList[i] = new(sql.NullInt32)
				fieldPointList[i] = fieldAddr
			case *int64:
				scanList[i] = new(sql.NullInt64)
				fieldPointList[i] = fieldAddr
			case *float32:
				scanList[i] = new(sql.NullFloat64)
				fieldPointList[i] = fieldAddr
			case *float64:
				scanList[i] = new(sql.NullFloat64)
				fieldPointList[i] = fieldAddr
			case *string:
				scanList[i] = new(sql.NullString)
				fieldPointList[i] = fieldAddr
			case *bool:
				scanList[i] = new(sql.NullBool)
				fieldPointList[i] = fieldAddr
			case *time.Time:
				scanList[i] = new(sql.NullTime)
				fieldPointList[i] = fieldAddr
			default: //指针类型数据可以直接扫描，无需处理
				scanList[i] = fieldAddr
			}
		}

		// 将当前行的数据扫描到scanList中
		if scanErr := rows.Scan(scanList...); scanErr != nil {
			LogUtil.Error(fmt.Sprintf("数据扫描失败:%s: err:%q", query, scanErr))
			return nil
		}
		for i, it := range fieldPointList {
			switch fieldPoint := it.(type) {
			case *int:
				nullValue := scanList[i].(*sql.NullInt64)
				if nullValue.Valid {
					*fieldPoint = int(nullValue.Int64)
				}
			case *int8:
				nullValue := scanList[i].(*sql.NullInt16)
				if nullValue.Valid {
					*fieldPoint = int8(nullValue.Int16)
				}
			case *int32:
				nullValue := scanList[i].(*sql.NullInt32)
				if nullValue.Valid {
					*fieldPoint = nullValue.Int32
				}
			case *int64:
				nullValue := scanList[i].(*sql.NullInt64)
				if nullValue.Valid {
					*fieldPoint = nullValue.Int64
				}
			case *float32:
				nullValue := scanList[i].(*sql.NullFloat64)
				if nullValue.Valid {
					*fieldPoint = float32(nullValue.Float64)
				}
			case *float64:
				nullValue := scanList[i].(*sql.NullFloat64)
				if nullValue.Valid {
					*fieldPoint = nullValue.Float64
				}
			case *string:
				nullValue := scanList[i].(*sql.NullString)
				if nullValue.Valid {
					*fieldPoint = nullValue.String
				}
			case *bool:
				nullValue := scanList[i].(*sql.NullBool)
				if nullValue.Valid {
					*fieldPoint = nullValue.Bool
				}
			case *time.Time:
				nullValue := scanList[i].(*sql.NullTime)
				if nullValue.Valid {
					*fieldPoint = nullValue.Time
				}
			}
		}
		dtoList = append(dtoList, *dtoT)
	}
	return dtoList
}

// SelectToListMap 将查询结果以List<Map>的类型返回
func SelectToListMap(query string, args ...any) []map[string]any {
	rows, err := getConnection().Query(query, args...)
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

// 通过反射的方式给结构体设置值
func setValue(field reflect.Value, value any) {
	kind := field.Kind()
	if kind == reflect.Ptr { //设置指针变量值
		ptr := reflect.New(field.Type().Elem())
		kindStr := field.Type().String()
		switch kindStr {
		case "*int32":
			ptr.Elem().Set(reflect.ValueOf(int32(value.(int64))))
		case "*int16":
			ptr.Elem().Set(reflect.ValueOf(int16(value.(int64))))
		case "*int8":
			ptr.Elem().Set(reflect.ValueOf(int8(value.(int64))))
		case "*int":
			ptr.Elem().Set(reflect.ValueOf(int32(value.(int64))))
		case "*float32":
			ptr.Elem().Set(reflect.ValueOf(float32(value.(float64))))
		default:
			ptr.Elem().Set(reflect.ValueOf(value))
		}
		field.Set(ptr)
	} else if kind == reflect.Int || kind == reflect.Int8 || kind == reflect.Int16 || kind == reflect.Int32 || kind == reflect.Int64 {
		field.SetInt(value.(int64))
	} else if kind == reflect.Float32 || kind == reflect.Float64 {
		field.SetFloat(value.(float64))
	} else {
		field.Set(reflect.ValueOf(value))
	}
}
