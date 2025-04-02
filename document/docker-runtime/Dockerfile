FROM ubuntu:24.10

RUN apt update

# 解决https更证书的问题
RUN apt-get update && apt-get install -y apt-transport-https ca-certificates
RUN update-ca-certificates


RUN mkdir /app
WORKDIR /app

#--------------------------------------------------------------------------------------------------------------安装LibRaw
#构建工具和依赖项
RUN yes|apt install zlib1g-dev
RUN yes|apt install pkg-config
RUN yes|apt install build-essential cmake autoconf

ADD https://github.com/DAIRO-HY/DairoDfsLib/raw/main/LibRaw-0.21.2-source.tar.gz /app/LibRaw-0.21.2-source.tar.gz
RUN tar -xzf LibRaw-0.21.2-source.tar.gz
WORKDIR /app/LibRaw-0.21.2
RUN ./configure
RUN make
RUN make install

#安装完成之后,执行dcraw_emu -v有可能报错
#dcraw_emu: error while loading shared libraries: libraw.so.23: cannot open shared object file: No such file or directory
#解决方案是执行ldconfig
RUN ldconfig

WORKDIR /app
RUN rm -rf LibRaw-0.21.2
RUN rm LibRaw-0.21.2-source.tar.gz


#---------------------------------------------------------------------------------------------------------安装ImageMagick
# pkg-config のパスを通す
RUN export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:/usr/local/lib64/pkgconfig:$PKG_CONFIG_PATH

# 動的ライブラリ（.soファイル）の検索パスを追加
RUN export LD_LIBRARY_PATH=/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH

RUN yes|apt-get install make
RUN yes|apt-get install cmake

RUN yes|apt-get install gcc
RUN yes|apt-get install gzip
RUN yes|apt-get install pkg-config
RUN yes|apt-get install libtool

# 安装g++
RUN yes|apt install build-essential

# 安装jpeg解码器
RUN yes|apt-get install libde265-dev libjpeg-dev libpng-dev
RUN yes|apt-get install libtiff-dev libwebp-dev
RUN yes|apt-get install libraw-dev


# 下载libheif源码
ADD https://github.com/strukturag/libheif/releases/download/v1.19.7/libheif-1.19.7.tar.gz libheif-1.19.7.tar.gz
RUN tar -xzvf libheif-1.19.7.tar.gz
WORKDIR /app/libheif-1.19.7

# 编译安装
RUN mkdir build
WORKDIR /app/libheif-1.19.7/build
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
WORKDIR /app
RUN rm libheif-1.19.7.tar.gz
RUN rm -rf libheif-1.19.7


ADD https://github.com/ImageMagick/ImageMagick/archive/refs/tags/7.1.1-45.tar.gz ImageMagick-7.1.1-45.tar.gz
RUN tar -xzvf ImageMagick-7.1.1-45.tar.gz
WORKDIR /app/ImageMagick-7.1.1-45
RUN ./configure
RUN make
RUN make install
RUN ldconfig /usr/local/lib
WORKDIR /app
RUN rm ImageMagick-7.1.1-45.tar.gz
RUN rm -rf ImageMagick-7.1.1-45

#下载源码并安装ExifTool
ADD https://exiftool.org/Image-ExifTool-13.26.tar.gz Image-ExifTool-13.26.tar.gz
RUN tar -xzvf Image-ExifTool-13.26.tar.gz
WORKDIR /app/Image-ExifTool-13.26
RUN apt-get install -y perl
RUN perl Makefile.PL
RUN make install
WORKDIR /app
RUN rm Image-ExifTool-13.26.tar.gz
RUN rm -rf Image-ExifTool-13.26



#清理操作
RUN apt-get remove -y make
RUN apt-get remove -y cmake
RUN apt-get remove -y gcc
RUN apt-get remove -y gzip
RUN apt-get remove -y autoconf
RUN apt-get remove -y build-essential
RUN apt-get remove -y perl
RUN apt autoremove -y



