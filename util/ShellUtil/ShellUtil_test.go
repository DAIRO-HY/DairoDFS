package ShellUtil

import (
	"fmt"
	"testing"
)

func TestExecToOkData2(t *testing.T) {
	data, err := ExecToOkData2("java -jar C:\\Users\\user\\IdeaProjects\\untitled3\\build\\libs\\untitled3-1.0-all.jar", []byte{1, 2, 3})
	if err != nil {
		fmt.Println(err)
	}
	fmt.Println("---------------------------")
	fmt.Println(len(data))
}
