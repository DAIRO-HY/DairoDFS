package CommonUtil

// 是否基本数据类型判断
func IsBaseType(value any) bool {
	switch value.(type) {
	case int, int8, int16, int32, int64,
		uint, uint8, uint16, uint32, uint64, uintptr,
		float32, float64,
		bool, string:
		return true
	default:
		return false
	}
}
