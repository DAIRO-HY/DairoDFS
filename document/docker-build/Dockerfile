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


#####################################################################安装docker开始#######################################
RUN mkdir /etc/docker
#COPY daemon.json /etc/docker
RUN curl -fsSL get.docker.com -o get-docker.sh

##若执行报错,请加参数 --mirror Aliyun
#RUN sh get-docker.sh --mirror Aliyun
RUN sh get-docker.sh

#安装docker-compose
RUN curl -SL https://github.com/docker/compose/releases/download/v2.24.0/docker-compose-linux-x86_64 -o /usr/local/bin/docker-compose
RUN chmod +x /usr/local/bin/docker-compose
####################################################################安装docker结束#######################################


WORKDIR /opt

#下载go
ADD https://go.dev/dl/go1.23.1.linux-amd64.tar.gz /opt

#解压
RUN tar -xzvf go1.23.1.linux-amd64.tar.gz

#删除安装包
RUN rm go1.23.1.linux-amd64.tar.gz

# 追加到PATH环境变量
ENV PATH="/opt/go/bin:${PATH}"


WORKDIR /home
COPY build-linux.sh build-linux.sh
RUN chmod +x build-linux.sh

#运行程序
#ENTRYPOINT "./build-linux.sh"
