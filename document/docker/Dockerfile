FROM ubuntu:24.10

RUN apt update

# 解决https更证书的问题
RUN apt install -y ca-certificates
RUN update-ca-certificates

RUN mkdir /app
WORKDIR /app

#下载程序
#ADD https://github.com/DAIRO-HY/DairoDFS/releases/download/2.0.0/dairo-dfs-linux-amd64 /app
COPY dairo-dfs-linux-amd64 /app

#授权可执行
RUN chmod +x dairo-dfs-linux-amd64

#启动脚本
ENTRYPOINT "/app/dairo-dfs-linux-amd64"