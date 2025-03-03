package String

import (
	"fmt"
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

func TestToShortString(t *testing.T) {
	fmt.Println(ToShortString(1879756789))
}

func TestMakeRandNumber(t *testing.T) {
	fmt.Println(MakeRandNumber(10))
}

func TestMakeRandStr(t *testing.T) {
	fmt.Println(MakeRandStr(32))
}
func TestFileExt(t *testing.T) {
	fmt.Println(FileExt("sfsdfsd.txt"))
}
