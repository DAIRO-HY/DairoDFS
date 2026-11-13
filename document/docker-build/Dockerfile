#编译环境镜像:只包含git、docker等工具
#Rust的编译在rust:1.99.1容器里进行(document/build/build_dfs.sh),arm64通过QEMU模拟编译
#Windows、macOS版在交叉编译容器里用cargo-zigbuild编译(document/build/build_dfs_cross.sh)
FROM ubuntu:24.04

RUN apt update

#安装curl、git、jq(build_github.sh解析GitHub接口返回的JSON需要)
RUN yes|apt install curl git jq ca-certificates

#####################################################################安装docker开始#######################################
#只用到docker命令和buildx插件,实际执行的是宿主机的docker(运行时挂载/var/run/docker.sock)
RUN curl -fsSL https://get.docker.com -o get-docker.sh

##若执行报错,请加参数 --mirror Aliyun
#RUN sh get-docker.sh --mirror Aliyun
RUN sh get-docker.sh && rm get-docker.sh
####################################################################安装docker结束#######################################

WORKDIR /home
COPY build-linux.sh build-linux.sh
RUN chmod +x build-linux.sh

#运行程序
ENTRYPOINT ["./build-linux.sh"]
