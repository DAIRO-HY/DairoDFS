#!/bin/bash


# 获取当前脚本文件的完整路径
SCRIPT_PATH=$(realpath "$0")

# 提取目录部分
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

# 进入当前目录
cd $SCRIPT_DIR

# pkg-config のパスを通す
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig:$PKG_CONFIG_PATH

# 動的ライブラリ（.soファイル）の検索パスを追加
export LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH

# yum でインストール可能な依存関係をインストール
apt-get update
yes|apt-get install make
yes|apt-get install cmake
yes|apt-get install curl


yes|apt-get install gcc
yes|apt-get install gzip
yes|apt-get install pkg-config
yes|apt-get install libtool

# 安装g++
yes|apt install build-essential

# 安装jpeg解码器
yes|apt-get install libde265-dev libjpeg-dev libpng-dev
yes|apt-get install libtiff-dev libwebp-dev
yes|apt-get install libraw-dev


# 下载libheif源码
curl -L -o libheif-1.19.7.tar.gz https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz
tar -xzvf libheif-1.19.7.tar.gz
cd libheif-1.19.7

# 编译安装
mkdir build
cd build
cmake --preset=release .. \
  -DCMAKE_BUILD_TYPE=Release \
  -DWITH_JPEG_DECODER=ON \
  -DWITH_JPEG_ENCODER=ON \
  -DWITH_LIBDE265=ON \
  -DWITH_X265=ON \
  -DWITH_PNG_DECODER=ON \
  -DWITH_PNG_ENCODER=ON
make
make install
cd ../..
rm libheif-1.19.7.tar.gz
rm -rf libheif-1.19.7


curl -L -o ImageMagick-7.1.1-45.tar.gz https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz
tar -xzvf ImageMagick-7.1.1-45.tar.gz
cd ImageMagick-7.1.1-45
./configure
make
make install
ldconfig /usr/local/lib
cd ..
rm ImageMagick-7.1.1-45.tar.gz
rm -rf ImageMagick-7.1.1-45