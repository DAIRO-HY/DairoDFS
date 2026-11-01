#!/bin/bash

apt-get update
apt-get install -y perl
yes|apt-get install make
yes|apt-get install curl

#官网只保留最新版本,旧版本的下载地址会失效,所以先获取最新的版本号
VERSION=$(curl -sL https://exiftool.org/ver.txt | tr -d '[:space:]')
if [ -z "$VERSION" ]; then
    echo "获取exiftool最新版本号失败"
    exit 1
fi
echo "exiftool最新版本:$VERSION"

#下载源码(官网的下载地址在SourceForge)
curl -fL -o Image-ExifTool-$VERSION.tar.gz https://downloads.sourceforge.net/project/exiftool/Image-ExifTool-$VERSION.tar.gz || exit 1
tar -xzvf Image-ExifTool-$VERSION.tar.gz || exit 1
cd Image-ExifTool-$VERSION

perl Makefile.PL
make
make install
