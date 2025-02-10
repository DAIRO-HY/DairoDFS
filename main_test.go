package main

import (
	"DairoDFS/controller/distributed"
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"testing"
	"time"
)

func TestMainTest(t *testing.T) {
	go tt1()
	go tt2()
	time.Sleep(1 * time.Hour)
}

func tt1() {
	ctx, cancel := context.WithCancel(context.Background())
	if cancel != nil {
		//cancel()
	}

	url := "http://localhost:8031/distributed/listen?lastId=0"
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)

	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: 3 * time.Second,                                     //读数据超时
	}
	client := &http.Client{Transport: transport}

	// 创建HTTP GET请求
	resp, _ := client.Do(req)
	if resp != nil {
		//resp.Body.Close()
	}
	//cancel()
	client.CloseIdleConnections()
}

func tt2() {
	ctx, cancel := context.WithCancel(context.Background())
	if cancel != nil {
		//cancel()
	}

	url := "http://localhost:8031/distributed/listen?lastId=0"
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)

	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: (distributed.KEEP_ALIVE_TIME + 10) * time.Second,    //读数据超时
	}
	client := &http.Client{Transport: transport}

	// 创建HTTP GET请求
	resp, _ := client.Do(req)
	if resp != nil {
		data, _ := io.ReadAll(resp.Body)
		fmt.Println(string(data))
	}
	client.CloseIdleConnections()
}
