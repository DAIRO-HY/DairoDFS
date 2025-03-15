FROM ubuntu:24.10

RUN apt update

#安装curl
RUN yes|apt install curl

#安装git
RUN yes|apt install git

#安装sqlite编译环境
RUN yes|apt-get install libsqlite3-dev

#安装gcc,编译sqlite需要
RUN yes|apt-get install gcc

WORKDIR /opt

#下载go
ADD https://go.dev/dl/go1.24.1.linux-amd64.tar.gz /opt

#解压
RUN tar -xzvf go1.24.1.linux-amd64.tar.gz

#删除安装包
RUN rm go1.24.1.linux-amd64.tar.gz

# 追加到PATH环境变量
ENV PATH="/opt/go/bin:${PATH}"