#运行时镜像(已经安装了LibRaw、ImageMagick、ExifTool),请先执行 build_dfs_runtime_docker.sh 构建并推送
ARG RUNTIME_IMAGE=dairopapa/dairo-dfs-runtime:latest
FROM ${RUNTIME_IMAGE}

ARG TARGETARCH

WORKDIR /app

COPY ./build/$TARGETARCH/DairoDFS /app/DairoDFS

#授权可执行
RUN chmod +x DairoDFS

#解决控制台输出中文乱码
ENV LANG=C.UTF-8

EXPOSE 8031

#数据保存在/app/data,请挂载到宿主机
ENTRYPOINT ["/app/DairoDFS", "--data-dir", "/app/data"]
