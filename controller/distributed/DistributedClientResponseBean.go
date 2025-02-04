package distributed

import "net/http"

/**
 * 分机端同步response信息
 */
type DistributedClientResponseBean struct {
	writer http.ResponseWriter

	clientToken string

	// 用来标记是否已经取消
	isCancel bool
}
