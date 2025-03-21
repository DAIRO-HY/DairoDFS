#请先构建镜像 ../docker-runtime/Dockerfile
FROM dairopapa/dairo-dfs-runtime:1.0.0
#FROM ubuntu:24.10

#下载程序
#ADD https://github.com/DAIRO-HY/DairoDFS/releases/download/2.0.0/dairo-dfs-linux-amd64 /app
COPY dairo-dfs-linux-amd64 /app

#授权可执行
RUN chmod +x dairo-dfs-linux-amd64

#启动脚本
ENTRYPOINT "/app/dairo-dfs-linux-amd64"