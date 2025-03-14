#FROM ubuntu:24.10
#RUN apt-get update
#RUN yes|apt install build-essential
#RUN yes|apt-get install libjpeg-dev libpng-dev libtiff-dev libwebp-dev libde265-dev libheif-dev libraw-dev
#ADD https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz temp.tar.gz
#RUN tar -xzvf temp.tar.gz
#WORKDIR /ImageMagick-7.1.1-45
#RUN ./configure --with-heic=yes
#RUN make
#RUN make install
#RUN ldconfig /usr/local/lib
#WORKDIR /home


#FROM amazonlinux:2
#
#WORKDIR /imagemagick
#
## pkg-config のパスを通す
#ENV PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig:$PKG_CONFIG_PATH
#
## 動的ライブラリ（.soファイル）の検索パスを追加
#ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH
#
## yum でインストール可能な依存関係をインストール
#RUN yum update -y && \
#    yum install -y \
#    gcc \
#    gcc-c++ \
#    git \
#    make \
#    cmake \
#    wget \
#    curl \
#    tar \
#    gzip \
#    pkg-config \
#    libtool \
#    openssl-devel \
#    libpng-devel \
#    libjpeg-turbo-devel \
#    gdk-pixbuf2-devel && \
#    yum clean all

# cmake をビルド&インストール
#RUN wget https://cmake.org/files/v3.27/cmake-3.27.0.tar.gz && \
#    tar -xvzf cmake-3.27.0.tar.gz && \
#    cd cmake-3.27.0 && \
#    ./bootstrap && \
#    make && \
#    make install
#
## libde265 をビルド&インストール
#RUN git clone https://github.com/strukturag/libde265.git -b v1.0.12 libde265 && \
#    cd libde265 && \
#    mkdir build && \
#    cd build && \
#    cmake .. && \
#    make && \
#    make install
#
#
#
## 下载libheif源码
#RUN curl -L -o 1.tar.gz https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz && \
#    tar -xzvf 1.tar.gz && \
#    cd libheif-1.19.7  && \
#    mkdir build  && \
#    cd build && \
#    cmake .. \
#      -DCMAKE_BUILD_TYPE=Release \
#      -DWITH_JPEG_DECODER=ON \
#      -DWITH_JPEG_ENCODER=ON \
#      -DWITH_LIBDE265=ON \
#      -DWITH_X265=ON \
#      -DWITH_PNG_DECODER=ON \
#      -DWITH_PNG_ENCODER=ON && \
#    make && \
#    make install
#

# libheif をビルド&インストール
#RUN git clone https://github.com/strukturag/libheif.git -b v1.16.2 libheif && \
#    cd libheif && \
#    mkdir build && \
#    cd build && \
#    cmake --preset=release .. && \
#    make && \
#    make install

# ImageMagick をビルド&インストール
#RUN wget https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz && \
#    tar -xzf ImageMagick-7.1.1-45.tar.gz && \
#    cd ImageMagick-7.1.1-45 && \
#    ./configure && \
#    make && \
#    make install















FROM ubuntu:24.10

#WORKDIR /imagemagick

# pkg-config のパスを通す
ENV PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig:$PKG_CONFIG_PATH

# 動的ライブラリ（.soファイル）の検索パスを追加
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH

# yum でインストール可能な依存関係をインストール
RUN apt-get update
#RUN apt-get install -y \
#    gcc \
#    gcc-c++ \
#    git \
RUN yes|apt-get install make
RUN yes|apt-get install cmake
RUN yes|apt-get install curl


RUN yes|apt-get install gcc
RUN #yes|apt-get install gcc-c++
RUN #yes|apt-get install git
RUN #yes|apt-get install make
RUN #yes|apt-get install cmake
RUN #yes|apt-get install wget
RUN #yes|apt-get install curl
RUN #yes|apt-get install tar
RUN yes|apt-get install gzip
RUN yes|apt-get install pkg-config
RUN yes|apt-get install libtool
RUN #yes|apt-get install openssl-dev
RUN #yes|apt-get install libpng-dev
RUN #yes|apt-get install libjpeg-turbo-dev
RUN #yes|apt-get install gdk-pixbuf2-dev

# 安装g++
RUN yes|apt install build-essential
#    tar \
#    gzip \
#    pkg-config \
#    libtool \
#    openssl-devel \
#    libpng-devel \
#    libjpeg-turbo-devel \
#    gdk-pixbuf2-devel \

# 安装jpeg解码器
RUN yes|apt-get install libde265-dev libjpeg-dev libpng-dev

# cmake をビルド&インストール
#RUN wget https://cmake.org/files/v3.27/cmake-3.27.0.tar.gz && \
#    tar -xvzf cmake-3.27.0.tar.gz && \
#    cd cmake-3.27.0 && \
#    ./bootstrap && \
#    make && \
#    make install

# libde265 をビルド&インストール
#RUN git clone https://github.com/strukturag/libde265.git -b v1.0.12 libde265 && \
#    cd libde265 && \
#    mkdir build && \
#    cd build && \
#    cmake .. && \
#    make && \
#    make install



# 下载libheif源码
RUN curl -L -o 1.tar.gz https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz
RUN tar -xzvf 1.tar.gz
WORKDIR /libheif-1.19.7

# 编译安装
RUN mkdir build
WORKDIR /libheif-1.19.7/build
#RUN yes|apt-get install cmake

# cmake .. -DCMAKE_BUILD_TYPE=Release  -DWITH_JPEG_DECODER=ON  -DWITH_JPEG_ENCODER=ON  -DWITH_LIBDE265=ON  -DWITH_X265=ON  -DWITH_PNG_DECODER=ON  -DWITH_PNG_ENCODER=ON
RUN cmake --preset=release .. \
  -DCMAKE_BUILD_TYPE=Release \
  -DWITH_JPEG_DECODER=ON \
  -DWITH_JPEG_ENCODER=ON \
  -DWITH_LIBDE265=ON \
  -DWITH_X265=ON \
  -DWITH_PNG_DECODER=ON \
  -DWITH_PNG_ENCODER=ON
RUN make
RUN make install


# libheif をビルド&インストール
#RUN git clone https://github.com/strukturag/libheif.git -b v1.16.2 libheif && \
#    cd libheif && \
#    mkdir build && \
#    cd build && \
#    cmake --preset=release .. && \
#    make && \
#    make install

# ImageMagick をビルド&インストール
#RUN wget https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz && \
#    tar -xzf ImageMagick-7.1.1-45.tar.gz && \
#    cd ImageMagick-7.1.1-45 && \
#    ./configure && \
#    make && \
#    make install

#RUN yes|apt-get install libjpeg-dev libpng-dev libtiff-dev libwebp-dev libde265-dev libheif-dev libraw-dev

WORKDIR /
ADD https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz temp.tar.gz
RUN tar -xzvf temp.tar.gz
WORKDIR /ImageMagick-7.1.1-45
RUN ./configure
RUN make
RUN make install
RUN ldconfig /usr/local/lib
WORKDIR /home