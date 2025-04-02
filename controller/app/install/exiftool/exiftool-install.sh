#!/bin/bash

apt-get update
apt-get install -y perl
yes|apt-get install make
yes|apt-get install curl

#下载源码
curl -L -o Image-ExifTool-13.26.tar.gz https://exiftool.org/Image-ExifTool-13.26.tar.gz
tar -xzvf Image-ExifTool-13.26.tar.gz
cd Image-ExifTool-13.26

perl Makefile.PL
make
make install