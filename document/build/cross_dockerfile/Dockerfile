#交叉编译环境:在Linux(amd64)上编译Windows、macOS的amd64和arm64版本(和DairoMUSIC、DairoMUSIC-Search相同)
#使用zig作为C编译器和链接器(ring、mimalloc、sqlite需要编译C代码),由cargo-zigbuild调用,不需要QEMU模拟
#macOS版本还需要macOS SDK(运行时挂载,见build_dfs_cross.sh)
FROM rust:1.99.1

ARG ZIG_VERSION=0.14.1
ARG ZIGBUILD_VERSION=0.24.1

RUN apt-get update && apt-get install -y --no-install-recommends xz-utils && rm -rf /var/lib/apt/lists/*

#安装zig
RUN curl -fsSL https://ziglang.org/download/${ZIG_VERSION}/zig-x86_64-linux-${ZIG_VERSION}.tar.xz -o /tmp/zig.tar.xz \
    && mkdir -p /opt/zig && tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 && rm /tmp/zig.tar.xz \
    && ln -s /opt/zig/zig /usr/local/bin/zig && zig version

#安装cargo-zigbuild
RUN cargo install cargo-zigbuild --version ${ZIGBUILD_VERSION} --locked && rm -rf /usr/local/cargo/registry

#安装编译目标
RUN rustup target add x86_64-pc-windows-gnu aarch64-pc-windows-gnullvm x86_64-apple-darwin aarch64-apple-darwin
