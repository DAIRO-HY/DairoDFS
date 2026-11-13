#!/bin/bash

#在Mac上执行:把macOS SDK打包成MacOSX.sdk.tar.gz,复制到Jenkins所在的Linux机器上解压,交叉编译macOS版本时使用
#Linux上交叉编译macOS程序需要SDK里的系统库和框架(比如CoreFoundation),SDK只能从Mac上获取
#用法: ./document/build/pack_macos_sdk.sh [输出目录]
#然后在Linux上: tar -xzf MacOSX.sdk.tar.gz -C /root/develop_tool  (解压后是 /root/develop_tool/MacOSX.sdk)

#遇到错误立即停止
set -e

out_dir=${1:-.}

#优先使用命令行工具里的SDK,没有时使用Xcode里的
sdk_path=$(xcrun --sdk macosx --show-sdk-path)
sdk_path=$(cd "$sdk_path" && pwd -P)
echo "SDK: $sdk_path"

#保留SDK里的符号链接,打包后的目录名统一为MacOSX.sdk
#不打包Mac专用的文件属性(fflags、xattr、._文件),否则Linux的GNU tar解压时会提示: 忽略未知的扩展头关键字‘SCHILY.fflags’
COPYFILE_DISABLE=1 tar --no-fflags --no-xattrs --no-mac-metadata -czf "$out_dir/MacOSX.sdk.tar.gz" -C "$(dirname "$sdk_path")" -s "|^$(basename "$sdk_path")|MacOSX.sdk|" "$(basename "$sdk_path")"
ls -lh "$out_dir/MacOSX.sdk.tar.gz"
