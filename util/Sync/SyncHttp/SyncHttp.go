package SyncHttp

import (
	"DairoDFS/exception"
	"DairoDFS/extension/String"
	"io"
	"net"
	"net/http"
	"time"
)

/**
 * 同步数据专用的HTTP请求工具类
 */

/**
 * 请求同步数据
 * @param url 请求url
 * @return 返回结果
 */
func Request(url string) ([]byte, error) {
	transport := &http.Transport{
		DialContext:           (&net.Dialer{Timeout: 3 * time.Second}).DialContext, //连接超时
		ResponseHeaderTimeout: 30 * time.Second,                                    //读数据超时
	}
	client := &http.Client{Transport: transport}

	// 创建HTTP GET请求
	resp, err := client.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		bodyData, _ := io.ReadAll(resp.Body)
		return nil, exception.Biz("Status: " + String.ValueOf(resp.StatusCode) + "  Body:" + string(bodyData))
	}
	data, _ := io.ReadAll(resp.Body)
	return data, nil
}
