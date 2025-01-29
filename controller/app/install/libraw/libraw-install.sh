#!/bin/bash

# 获取当前脚本文件的完整路径
SCRIPT_PATH=$(realpath "$0")

# 提取目录部分
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

# 进入当前目录
cd $SCRIPT_DIR

apt update

#构建工具和依赖项
yes|apt install build-essential cmake

curl -L -o LibRaw-0.21.2-source.tar.gz https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-source.tar.gz

tar -xzf LibRaw-0.21.2-source.tar.gz

cd LibRaw-0.21.2

#构建和安装 LibRaw
./configure
make
make install

#安装完成之后,执行dcraw_emu -v有可能报错
#dcraw_emu: error while loading shared libraries: libraw.so.23: cannot open shared object file: No such file or directory
#解决方案是执行ldconfig
ldconfig

#删除安装包
rm ../LibRaw-0.21.2-source.tar.gz