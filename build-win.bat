@echo off
chcp 65001 >nul

rem 最终编译的二进制文件名
set EXEC_FILE="./dairo-dfs-win-amd64.exe"

rem ---------------------------------------开始编译-----------------------------------------
set CGO_ENABLED=1
go build -ldflags="-s -w" -o %EXEC_FILE%
echo "编译完成"

pause
