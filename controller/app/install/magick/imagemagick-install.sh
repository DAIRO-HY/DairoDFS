#!/bin/bash

# pkg-config のパスを通す
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig:$PKG_CONFIG_PATH

# 動的ライブラリ（.soファイル）の検索パスを追加
export LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH

# yum でインストール可能な依存関係をインストール
apt-get update
#apt-get install -y \
#    gcc \
#    gcc-c++ \
#    git \
yes|apt-get install make
yes|apt-get install cmake
yes|apt-get install curl


yes|apt-get install gcc
#yes|apt-get install gcc-c++
#yes|apt-get install git
yes|apt-get install gzip
yes|apt-get install pkg-config
yes|apt-get install libtool

# 安装g++
yes|apt install build-essential
#    pkg-config \
#    libtool \

# 安装jpeg解码器
yes|apt-get install libde265-dev libjpeg-dev libpng-dev
yes|apt-get install libtiff-dev libwebp-dev
#yes|apt-get install libheif-dev
yes|apt-get install libraw-dev
#yes|apt-get install dcraw



# 下载libheif源码
curl -L -o 1.tar.gz https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz
tar -xzvf 1.tar.gz
cd ./libheif-1.19.7

# 编译安装
mkdir build
cd ./build
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
curl -L -o temp.tar.gz https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz
tar -xzvf temp.tar.gz
cd ImageMagick-7.1.1-45
./configure
make
make install
ldconfig /usr/local/lib
cd /home