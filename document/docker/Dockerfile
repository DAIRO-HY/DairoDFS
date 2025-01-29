FROM frolvlad/alpine-glibc:glibc-2.34

RUN mkdir /app
WORKDIR /app

#下载程序
#ADD https://github.com/DAIRO-HY/DairoNPS/releases/download/1.0.0/dairo-nps-linux-amd64 /app
COPY dairo-nps-linux-amd64 /app

#授权可执行
RUN chmod +x dairo-nps-linux-amd64


#后台管理用户
ENV NPS_LOGIN_NAME="admin"

#后台管理密码
ENV NPS_LOGIN_PWD=""

#后台管理端口
ENV NPS_WEB_PORT=1780

#客户端连接TCP端口
ENV NPS_TCP_PORT=1781

#客户端连接UDP端口
ENV NPS_UDP_PORT=1782

#日志类型
ENV NPS_LOG_TYPE=2

#日志级别
ENV NPS_LOG_LEVEL="error"

#启动脚本
ENTRYPOINT "/app/dairo-nps-linux-amd64" -login-name:$NPS_LOGIN_NAME -login-pwd:$NPS_LOGIN_PWD -web-port:$NPS_WEB_PORT -tcp-port:$NPS_TCP_PORT -udp-port:$NPS_UDP_PORT -log-type:$NPS_LOG_TYPE -log-level:$NPS_LOG_LEVEL