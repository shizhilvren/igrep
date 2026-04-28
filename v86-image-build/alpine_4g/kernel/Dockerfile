# 使用 Alpine 最新版本作为基础镜像
FROM i386/alpine:3.23

# 设置环境变量
ENV PACKAGER="Alpine Builder <builder@alpinelinux.org>"

# =====================================================
# 阶段 1: Root 用户操作 (安装依赖、创建用户)
# =====================================================

# 1. 安装基础依赖
RUN apk update
RUN apk add --no-cache git alpine-sdk abuild ccache 
RUN apk add --no-cache ncurses-dev openssl-dev perl-utils 
RUN apk add --no-cache kexec-tools linux-lts ripgrep 
RUN apk add --no-cache build-base sudo
RUN apk add --no-cache flex bison build-base


# 2. 创建 abuild 用户 (关键：必须赋予 sudo 权限)
RUN adduser -D -G abuild abuild  
RUN echo '%abuild ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# =====================================================
# 阶段 2: abuild 用户操作 (拆分为独立命令)
# =====================================================
# 切换到 abuild 用户
USER abuild
WORKDIR /home/abuild

# 2.2 克隆 aports 仓库
RUN cd /home/abuild && git clone https://git.alpinelinux.org/aports.git --depth=1

# 2.1 生成构建密钥
RUN mkdir -p /home/abuild/.abuild

RUN abuild-keygen -a -n
RUN abuild-keygen -n -i 
RUN abuild-keygen -n  --kernel

USER root
RUN cat /home/abuild/.abuild/abuild.conf >> /etc/abuild.conf
USER abuild

# 2.3 切换到自定义分支
WORKDIR /home/abuild/aports
RUN git switch master
RUN git branch -d my-virt-build || true
RUN git checkout -b my-virt-build master

# 2.4 准备自定义内核配置
WORKDIR /home/abuild/aports/main/linux-lts
RUN cp virt.x86.config virt.my_custom.x86.config
RUN echo "CONFIG_HIGHMEM4G=y" >> virt.my_custom.x86.config

# 2.5 生成基础校验和
RUN abuild checksum

# 2.6 更新 FLAVOR 相关校验和
RUN FLAVOR=virt.my_custom abuild checksum

# 2.7 执行内核构建 (核心步骤)
RUN FLAVOR=virt.my_custom abuild -r

# 2.8 验证构建结果
RUN ls -lh /home/abuild/packages/main/x86/linux-lts-my_custom-*.apk 2>/dev/null || \
    echo "⚠️ 未找到构建包，请检查错误日志"

# =====================================================
# 阶段 3: 保持容器运行
# =====================================================
USER root
RUN apk add --no-cache procps
CMD ["tail", "-f", "/dev/null"]