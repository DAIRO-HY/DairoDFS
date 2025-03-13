#!/bin/bash

# 获取当前脚本文件的完整路径
SCRIPT_PATH=$(realpath "$0")

# 提取目录部分
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

# 进入当前目录
cd $SCRIPT_DIR

apt-get update

# 安装jpeg解码器
yes|apt-get install libde265-dev libjpeg-dev libpng-dev

# 安装g++
yes|apt install build-essential
yes|apt-get install curl

# 下载libheif源码
curl -L -o 1.tar.gz https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz
tar -xzvf 1.tar.gz
cd libheif-1.19.7/

# 编译安装
mkdir build
cd build
yes|apt-get install cmake

# cmake .. -DCMAKE_BUILD_TYPE=Release  -DWITH_JPEG_DECODER=ON  -DWITH_JPEG_ENCODER=ON  -DWITH_LIBDE265=ON  -DWITH_X265=ON  -DWITH_PNG_DECODER=ON  -DWITH_PNG_ENCODER=ON
cmake .. \
  -DCMAKE_BUILD_TYPE=Release \
  -DWITH_JPEG_DECODER=ON \
  -DWITH_JPEG_ENCODER=ON \
  -DWITH_LIBDE265=ON \
  -DWITH_X265=ON \
  -DWITH_PNG_DECODER=ON \
  -DWITH_PNG_ENCODER=ON

make
