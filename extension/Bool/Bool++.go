package Bool

// 模拟三元表达式
func Is[T any](flag bool, trueValue T, falseValue T) T {
	if flag {
		return trueValue
	} else {
		return falseValue
	}
}
