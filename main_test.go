package main

import (
	"DairoDFS/test"
	"fmt"
	"runtime"
	"testing"
	"time"
)

const count = 10000

var now int64

func TestMainTest(t *testing.T) {
	//var temp int64 = 10
	//ttemp := &temp
	//ttempp := &ttemp
	//fmt.Println(ttempp)
	//return

	//makeStruct()
	//makeStructPoint()
	//makeStructPointPoint()
	//return

	//var list []Test100000.Test
	//for i := 0; i < 10; i++ {
	//	list = append(list, Test100000.Test{})
	//}
	//time.Sleep(10 * time.Second)
	//fmt.Printf("mem-size = %dKB\n", unsafe.Sizeof(list)/1024)
	//fmt.Println(len(list))
	//return
	//fmt.Printf("mem-size = %dKB\n", unsafe.Sizeof(Test10000.Test{})/1024)
	//

	// 获取内存使用情况
	var memStats runtime.MemStats
	for i := 0; i < 5; i++ {
		if i == 4 {
			go byPoint(1)
		} else {
			go byPoint(count)
		}
		time.Sleep(1 * time.Second)
		runtime.GC()
		fmt.Printf("NumGC: %d //垃圾回收次数\n", memStats.NumGC)
		fmt.Println("GC FINISH")
	}
	fmt.Println("-------------->FINISH")
	time.Sleep(1000 * time.Second)
	//byValue()
	//
	//byPoint()
	//byValue()
	//
	////runtime.GC()
	//
	//runtime.ReadMemStats(&memStats)
	//fmt.Printf("NumGoroutine: %d //当前协程数\n", runtime.NumGoroutine())
	//fmt.Printf("Memory: %s //内存分配\n", Number.ToDataSize(memStats.Alloc))
	//fmt.Printf("SystemMemory: %s //系统内存占用\n", Number.ToDataSize(memStats.Sys))
	//fmt.Printf("HeapAlloc: %s //堆内存分配\n", Number.ToDataSize(memStats.HeapAlloc))
	//fmt.Printf("HeapSys: %s //堆内存系统占用\n", Number.ToDataSize(memStats.HeapSys))
	fmt.Printf("NumGC: %d //垃圾回收次数\n", memStats.NumGC)
}

func byPoint(count int) {
	//var list []test.TestPP
	now = time.Now().UnixMilli()
	for i := 0; i < count; i++ {
		tp := test.InitPP()
		//list = append(list, tp)
		//time.Sleep(1 * time.Second)
		if tp.I0000 == time.Now().UnixMilli() {
			fmt.Println("OK")
		}
	}
	times := time.Now().UnixMilli() - now
	fmt.Printf("timr-point-总 = %d毫秒\n", times)
	fmt.Printf("timr-point-均 = %.10f毫秒\n\n", float64(times)/float64(count))
	//for _, it := range list {
	//	it.Clear()
	//}
	//list = nil
}

//func byValue() {
//	now = time.Now().UnixMilli()
//	for i := 0; i < count; i++ {
//		//go
//		func() {
//			var t Test10000.Test
//			t.HandelByValue()
//			//time.Sleep(10000 * time.Second)
//		}()
//	}
//	times := time.Now().UnixMilli() - now
//	fmt.Printf("timr-value-总 = %d毫秒\n", times)
//	fmt.Printf("timr-value-均 = %.10f毫秒\n\n", float64(times)/count)
//}
