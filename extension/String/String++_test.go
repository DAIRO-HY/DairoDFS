package String

import (
	"testing"
)

// 获取上级目录路径
func TestGetParentPath1(t *testing.T) {
	parent := FileParent("/abc/def/hij.txt")
	if parent != "/abc/def" {
		t.Error("失败")
	}
}

// 获取上级目录路径
func TestGetParentPath2(t *testing.T) {
	parent := FileParent("hij.txt")
	if parent != "" {
		t.Error("失败")
	}
}

// 获取上级目录路径
func TestGetParentPath3(t *testing.T) {
	parent := FileParent("./hij.txt")
	if parent != "." {
		t.Error("失败")
	}
}

// 获取上级目录路径
func TestGetParentPath4(t *testing.T) {
	parent := FileParent("\\abc\\def\\hij.txt")
	if parent != "\\abc\\def" {
		t.Error("失败")
	}
}

// 获取上级目录路径
func TestGetParentPath5(t *testing.T) {
	parent := FileParent("\\abc/def\\hij.txt")
	if parent != "\\abc/def" {
		t.Error("失败")
	}
}
