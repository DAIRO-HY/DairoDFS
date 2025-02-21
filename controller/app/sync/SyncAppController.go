package sync

import (
	"DairoDFS/controller/app/sync/form"
	"DairoDFS/extension/Bool"
	"DairoDFS/extension/Date"
	"DairoDFS/util/DistributedUtil/SyncByLog"
	"DairoDFS/util/DistributedUtil/SyncByTable"
	"DairoDFS/util/DistributedUtil/SyncInfoManager"
	"encoding/json"
	"fmt"
	"github.com/gorilla/websocket"
	"net/http"
	"reflect"
	"time"
)

// 数据同步状态
//@Group: /app/sync

/**
 * 页面初始化
 */
//@Html:.html
func Html() {}

/**
 * 页面数据初始化
 */
//@Post:/info_list
func InfoList() []form.SyncServerForm {
	formList := make([]form.SyncServerForm, 0)
	for _, it := range SyncInfoManager.SyncInfoList {
		outForm := form.SyncServerForm{
			Url:           it.Url,
			State:         it.State,
			Msg:           it.Msg,
			No:            it.No,
			SyncCount:     it.SyncCount,
			LastHeartTime: Bool.Is(it.LastHeartTime == 0, "无", Date.FormatByTimespan(it.LastHeartTime)),
			LastTime:      Bool.Is(it.LastTime == 0, "无", Date.FormatByTimespan(it.LastTime)),
		}
		formList = append(formList, outForm)
	}
	return formList
}

// 日志同步
// @Post:/by_log
func BySync() {
	SyncByLog.ListenAll()
}

// 全量同步
// @Post:/by_table
func ByTable() {
	go SyncByTable.SyncAll()
}

// 当前同步状态
// @Request:/info
func Info(writer http.ResponseWriter, request *http.Request) {

	// 创建WebSocket升级器
	var upgrader = websocket.Upgrader{
		// 允许跨域请求
		CheckOrigin: func(r *http.Request) bool {
			return true
		},
	}

	// 将HTTP连接升级为WebSocket连接
	conn, err := upgrader.Upgrade(writer, request, nil)
	if err != nil {
		fmt.Println("升级为WebSocket失败:", err)
		return
	}
	defer conn.Close()

	//记录上次发送的数据，如果前后两次发送的数据一样，则不要发送数据
	var preList []form.SyncServerForm
	for {
		time.Sleep(500 * time.Millisecond)
		infoList := InfoList()             //获取下载信息
		if len(preList) != len(infoList) { //初始化上次请求的数据
			for i := 0; i < len(infoList); i++ {
				preList = append(preList, form.SyncServerForm{})
			}
		}

		//返回给前端的数据，有变化的下标对应的数据Map
		outFormIndexMap := make(map[int]form.SyncServerForm)
		for x, it := range infoList {
			outForm := new(form.SyncServerForm)
			preIt := preList[x]
			itReflect := reflect.ValueOf(&it).Elem()
			preItReflect := reflect.ValueOf(&preIt).Elem()
			outItReflect := reflect.ValueOf(outForm).Elem()

			//标记本行是否有变更
			hasDiff := false
			for i := 0; i < itReflect.NumField(); i++ {
				if itReflect.Field(i).Interface() != preItReflect.Field(i).Interface() { //和上一次的值进行比较，不一样的值才往前端返回
					value := itReflect.Field(i).Interface()
					outItReflect.Field(i).Set(reflect.ValueOf(value))
					hasDiff = true
				}
			}
			if hasDiff {
				outForm.State = it.State
				outFormIndexMap[x] = *outForm
			}
		}
		if len(outFormIndexMap) == 0 { //没有数据变化
			if conn.WriteMessage(websocket.TextMessage, []byte("0")) != nil {
				break
			}
			continue
		}
		preList = infoList
		jsonData, _ := json.Marshal(outFormIndexMap)

		// 发送消息
		if conn.WriteMessage(websocket.TextMessage, jsonData) != nil {
			break
		}
	}
}
