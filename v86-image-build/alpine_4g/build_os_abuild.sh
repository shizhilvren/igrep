# 2. 设置 abuild 环境 (非交互式)
# 注意：你需要先运行 `abuild-keygen -a` 生成密钥，或确保有签名密钥
if [ ! -f /home/abuild/.abuild/default.rsa ]; then
    echo "🔑 检测到无构建密钥，正在生成..."
    mkdir -p /home/abuild/.abuild
    # 这里使用默认邮箱，生产环境请修改
    echo "PACKAGER_PRIVKEY=\"/home/abuild/.abuild/default.rsa\"" >> /etc/abuild.conf
    abuild-keygen -a -n
fi

# 3. 克隆 aports 仓库
echo "📂 正在克隆 Alpine Ports 仓库..."
cd /root || exit
if [ ! -d "./aports" ]; then
    git clone https://git.alpinelinux.org/aports.git
fi
cd aports || exit

# --- 定制点 1: 选择分支 ---
# 根据你的 Alpine 版本修改这里 (例如 3.23 或 master)
BRANCH="master" 
git switch master
git branch -d my-virt-build || true
git checkout -b my-virt-build $BRANCH

# 4. 进入 linux-lts 目录
cd main/linux-lts || exit

# --- 定制点 2: 准备自定义配置 ---
# 基于现有的 virt 配置创建副本
cp virt.x86.config virt.my_custom.x86.config
echo "CONFIG_HIGHMEM4G=y" >>  virt.my_custom.x86.config
rg HIGHMEM
# 或者，如果你想基于当前运行的系统生成最小配置，取消注释下一行：
# zcat /proc/config.gz > .config



# 更新校验和
CHOST=x86 CTARGET=x86 abuild -F checksum

echo "🔧 配置阶段：现在你可以编辑 virt.my_custom.x86_64.config"
echo "   或者脚本将启动 menuconfig 供你交互式配置。"
echo "   按回车键继续进入配置界面（或 Ctrl+C 退出进行手动修改）..."
read

echo "✅ 配置已保存。正在更新校验和并准备构建..."
cd /root/aports/main/linux-lts/
FLAVOR=virt.my_custom  CHOST=x86 CTARGET=x86 abuild -F checksum

# 6. 构建内核
# --- 定制点 3: 设置 FLAVOR ---
# 这里的 FLAVOR 名称必须与你的配置文件名 (virt.my_custom) 对应
echo "🔨 正在构建内核... (这可能需要数小时，请耐心等待)"
FLAVOR=virt.my_custom CHOST=x86 CTARGET=x86 abuild -F -r

echo "🎉 构建完成！"
echo "💡 提示：安装包位于 ~/packages/main/x86_64/ 目录下。"
echo "💡 安装命令示例: apk add ~/packages/main/x86_64/linux-lts-my_custom-*.apk"