#请先构建镜像 ../docker-runtime/Dockerfile
FROM dairo-dfs-runtime
#FROM ubuntu:24.10

#下载程序
#ADD https://github.com/DAIRO-HY/DairoDFS/releases/download/2.0.0/dairo-dfs-linux-amd64 /app
COPY dairo-dfs-linux-amd64 /app

#授权可执行
RUN chmod +x dairo-dfs-linux-amd64

#启动脚本
ENTRYPOINT ["/app/dairo-dfs-linux-amd64","--log-out-type","2","--log-level","debug,info,error"]