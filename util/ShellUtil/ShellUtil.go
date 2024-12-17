package ShellUtil

import (
	"io"
	"os/exec"
	"strings"
)

// 将执行结果输出到正常流
// - command 指令
// - okReader 正常数据流
// return 错误结果，error
func ExecToOkReader(command string, okReader func(io.ReadCloser)) (string, error) {
	var errResult string
	errReader := func(reader io.ReadCloser) {
		var data []byte
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			data = append(data[:], buf[:n]...)
		}
		errResult = string(data)
	}
	err := ExecToReader(command, okReader, errReader)
	return errResult, err
}

// 将执行结果输出到错误流
// - command 指令
// - errReader 错误数据流
// return 正常结果，error
func ExecToErrReader(command string, errReader func(io.ReadCloser)) (string, error) {
	var okResult string
	reader := func(reader io.ReadCloser) {
		var data []byte
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			data = append(data[:], buf[:n]...)
		}
		okResult = string(data)
	}
	err := ExecToReader(command, reader, errReader)
	return okResult, err
}

// 将执行结果输出到字符串
// - command 指令
// return 正常结果，异常结果，error
func ExecToResult(command string) (string, string, error) {
	var okResult string
	reader := func(reader io.ReadCloser) {
		var data []byte
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			data = append(data[:], buf[:n]...)
		}
		okResult = string(data)
	}

	var errResult string
	errReader := func(reader io.ReadCloser) {
		var data []byte
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			data = append(data[:], buf[:n]...)
		}
		errResult = string(data)
	}
	err := ExecToReader(command, reader, errReader)
	return okResult, errResult, err
}

// 将执行结果输出到流
// - command 指令
// - okReader 正常数据流
// - errReader 错误数据流
// return error
func ExecToReader(command string, reader func(io.ReadCloser), errReader func(io.ReadCloser)) error {
	cmdArr := parseCmd(command)
	cmd := exec.Command(cmdArr[0], cmdArr[1:]...)
	stdout, err := cmd.StdoutPipe()
	defer stdout.Close()
	stderr, err := cmd.StderrPipe()
	defer stderr.Close()

	if err != nil {
		return err
	}
	err = cmd.Start()
	if err != nil {
		return err
	}
	go reader(stdout)
	errReader(stderr)
	err = cmd.Wait()
	if err != nil {
		return err
	}
	return nil
}

// 去解析指令
func parseCmd(command string) []string {
	var cmds []string
	var cmdTemp = command + " "
	for len(cmdTemp) != 0 {
		var nextIndex int
		if cmdTemp[0] == '"' { //如果指令有使用双引号
			nextIndex = strings.Index(cmdTemp, "\" ")
			cmds = append(cmds, cmdTemp[1:nextIndex])
			cmdTemp = cmdTemp[nextIndex+2:]
		} else {
			nextIndex = strings.Index(cmdTemp, " ")
			cmds = append(cmds, cmdTemp[0:nextIndex])
			cmdTemp = cmdTemp[nextIndex+1:]
		}
	}
	//return cmdList.filter{it.isNotEmpty()}
	return cmds
}
