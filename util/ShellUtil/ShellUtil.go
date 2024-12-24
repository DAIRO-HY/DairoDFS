package ShellUtil

import (
	"DairoDFS/exception"
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

// 将执行成功的结果返回
// - command 指令
// return 正常结果，异常结果，error
func ExecToOkResult(command string) (string, error) {
	okData, err := ExecToOkData(command)
	return string(okData), err
}

// 将执行成功的结果输出到字节数组
// - command 指令
// return 正常数据，异常数据，error
func ExecToOkData(command string) ([]byte, error) {
	okData, errData, CmdErr := ExecToOkAndErrorData(command)
	if CmdErr != nil { //如果执行出错
		if len(errData) > 0 {
			return nil, exception.Biz(string(errData))
		} else {
			return nil, CmdErr
		}
	}
	return okData, nil
}

// 将执行成功的结果返回
// - command 指令
// return 正常结果，异常结果，error
func ExecToOkAndErrorResult(command string) (string, string, error) {
	okData, errData, err := ExecToOkAndErrorData(command)
	return string(okData), string(errData), err
}

// 将执行结果输出字节数组中
// 如果成功数据流没有数据，将会返回错误数据流中的数据
// - command 指令
// return 正常数据，异常数据，error
func ExecToOkAndErrorData(command string) ([]byte, []byte, error) {
	var okData []byte
	reader := func(reader io.ReadCloser) {
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			okData = append(okData[:], buf[:n]...)
		}
	}

	var errData []byte
	errReader := func(reader io.ReadCloser) {
		buf := make([]byte, 8*1024)
		for {
			n, err := reader.Read(buf)
			if err != nil && err == io.EOF {
				break
			}
			errData = append(errData[:], buf[:n]...)
		}
	}
	err := ExecToReader(command, reader, errReader)
	return okData, errData, err
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
