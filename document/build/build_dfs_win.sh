#!/bin/bash

#遇到错误立即停止
set -e

# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

#先创建文件夹
mkdir -p $project_root/build

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/DairoDFS/Cargo.toml)

#qemu主机的ip和ssh端口
QEMU_USER=${QEMU_USER:-root}
QEMU_HOST=${QEMU_HOST:-192.168.3.202}
QEMU_PORT=${QEMU_PORT:-22}

#windows系统的ip和ssh端口
WINDOWS_USER=${WINDOWS_USER:-Administrator}
WINDOWS_HOST=${WINDOWS_HOST:-192.168.3.202}
WINDOWS_PORT=${WINDOWS_PORT:-2201}

#发送开机指令
ssh -p $QEMU_PORT $QEMU_USER@$QEMU_HOST "qemu-system-x86_64 \
      -enable-kvm \
      -m 6G \
      -smp 8 \
      -cpu host \
      -drive file=/home/qemu/virtio-win-0.1.285.iso,media=cdrom \
      -drive file=/home/qemu/windows-server-2022.qcow2,if=virtio \
      -netdev user,id=net0,hostfwd=tcp::2201-:22,hostfwd=tcp::3389-:3389 \
      -device e1000,netdev=net0 \
      -display none \
      -daemonize" || echo "虚拟机可能已经启动"

#等待开机完成
wait_windows_start(){
  RETRYS=0
  while ! echo >/dev/tcp/$WINDOWS_HOST/$WINDOWS_PORT 2>/dev/null
  do
      echo "等待 SSH 端口开放..."
      RETRYS=$((RETRYS+1))
      echo "重试次数: $RETRYS"
      if [ $RETRYS -gt 30 ]; then
          echo "等待 SSH 端口开放超时，退出脚本。"
          exit 1
      fi
      sleep 1
  done
  echo "SSH 已上线"
}

#等待windows系统开机完成
wait_windows_start

#将编译脚本发送到windows系统
scp -P $WINDOWS_PORT $script_dir/build_dfs_win.bat $WINDOWS_USER@$WINDOWS_HOST:./
ssh -p $WINDOWS_PORT $WINDOWS_USER@$WINDOWS_HOST "build_dfs_win.bat"
scp -P $WINDOWS_PORT $WINDOWS_USER@$WINDOWS_HOST:develop/DairoDFS/target/release/DairoDFS.exe $project_root/build/DairoDFS-win-amd64-$version.exe

# 编译结束之后立即关机
#ssh -p $WINDOWS_PORT $WINDOWS_USER@$WINDOWS_HOST "shutdown /s /t 0"
