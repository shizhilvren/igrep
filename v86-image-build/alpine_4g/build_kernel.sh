#!/bin/sh -x
# =====================================================
# Alpine Linux 自定义 linux-lts 内核构建脚本
# 基于 Alpine Wiki Custom_Kernel 指南定制
# =====================================================

set -e # 遇到错误立即停止

echo "🚀 开始 Alpine Linux Virt 内核定制流程..."

# 1. 安装基础依赖
echo "📦 正在安装构建依赖..."
apk add --no-cache git alpine-sdk abuild ccache \
    ncurses-dev openssl-dev perl-utils \
    kexec-tools linux-lts ripgrep vim \
    flex bison build-base fd ncurses-dev
# 强烈建议保留 lts 作为救援内核

adduser abuild
