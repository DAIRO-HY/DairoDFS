# ubuntu 24.10已经停止维护,apt源已经下线,改用长期支持版本
FROM ubuntu:24.04

# 安装过程中不要弹出交互提示(比如时区选择)
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update

# 解决https证书的问题
RUN apt install -y ca-certificates curl
RUN update-ca-certificates

# 时区数据,通过环境变量TZ指定时区(照片的拍摄时间按服务器时区计算)
RUN apt install -y tzdata


RUN mkdir /app
WORKDIR /app

#--------------------------------------------------------------------------------------------------------------安装LibRaw
#构建工具和依赖项
RUN apt install -y zlib1g-dev pkg-config build-essential cmake autoconf

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
RUN apt install -y make gcc gzip libtool

# 安装jpeg等解码器
RUN apt install -y libde265-dev libjpeg-dev libpng-dev libtiff-dev libwebp-dev libraw-dev

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

#---------------------------------------------------------------------------------------------------------安装ExifTool
#官网只保留最新版本,旧版本的下载地址会失效,所以先获取最新的版本号再下载(下载地址在SourceForge)
RUN apt install -y perl
RUN VERSION=$(curl -sL https://exiftool.org/ver.txt | tr -d '[:space:]') \
  && curl -fL -o Image-ExifTool.tar.gz https://downloads.sourceforge.net/project/exiftool/Image-ExifTool-$VERSION.tar.gz \
  && tar -xzf Image-ExifTool.tar.gz \
  && cd Image-ExifTool-$VERSION \
  && perl Makefile.PL \
  && make install \
  && cd .. \
  && rm -rf Image-ExifTool.tar.gz Image-ExifTool-$VERSION

# 清理apt缓存,减小镜像体积
RUN rm -rf /var/lib/apt/lists/*
