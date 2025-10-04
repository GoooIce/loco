#!/bin/bash
# Loco MCP 完整安装脚本（包含虚拟环境设置和 Claude Desktop 配置）

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$PROJECT_ROOT/.venv"

echo "=================================="
echo "Loco MCP 完整安装向导"
echo "=================================="
echo

# 步骤 1: 设置虚拟环境
echo "步骤 1: 设置虚拟环境"
echo "----------------------------"
"$PROJECT_ROOT/setup_venv.sh"

# 激活虚拟环境
source "$VENV_DIR/bin/activate"

# 步骤 2: 配置 Claude Desktop
echo
echo "步骤 2: 配置 Claude Desktop"
echo "----------------------------"
echo

# 检测操作系统
OS="$(uname -s)"
case "${OS}" in
    Darwin*)    MACHINE=Mac;;
    Linux*)     MACHINE=Linux;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

if [ "$MACHINE" = "Mac" ]; then
    CONFIG_DIR="$HOME/Library/Application Support/Claude"
    CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
    LOG_DIR="$HOME/Library/Logs/Claude"
elif [ "$MACHINE" = "Linux" ]; then
    CONFIG_DIR="$HOME/.config/Claude"
    CONFIG_FILE="$CONFIG_DIR/claude_desktop_config.json"
    LOG_DIR="$HOME/.cache/Claude/logs"
else
    echo "未知操作系统，请手动配置 Claude Desktop"
    exit 0
fi

echo "配置文件位置: $CONFIG_FILE"

# 创建配置目录
mkdir -p "$CONFIG_DIR"

# 使用虚拟环境中的 Python
VENV_PYTHON="$VENV_DIR/bin/python3"

# 生成配置内容
CONFIG_CONTENT=$(cat <<EOF
{
  "mcpServers": {
    "loco": {
      "command": "$VENV_PYTHON",
      "args": ["-m", "loco_mcp_server.server"]
    }
  }
}
EOF
)

# 检查配置文件是否存在
if [ -f "$CONFIG_FILE" ]; then
    echo "⚠️  配置文件已存在"
    echo "   当前配置:"
    cat "$CONFIG_FILE"
    echo
    echo "   建议的配置:"
    echo "$CONFIG_CONTENT"
    echo
    read -p "是否覆盖现有配置？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 备份现有配置
        cp "$CONFIG_FILE" "$CONFIG_FILE.backup.$(date +%Y%m%d_%H%M%S)"
        echo "$CONFIG_CONTENT" > "$CONFIG_FILE"
        echo "✅ 配置已更新（旧配置已备份）"
    else
        echo "ℹ️  配置未更改"
        echo "   请手动添加以下内容到 $CONFIG_FILE:"
        echo "$CONFIG_CONTENT"
    fi
else
    echo "$CONFIG_CONTENT" > "$CONFIG_FILE"
    echo "✅ 配置文件已创建"
fi

# 完成
echo
echo "=================================="
echo "🎉 安装完成！"
echo "=================================="
echo
echo "虚拟环境: $VENV_DIR"
echo "Python: $VENV_PYTHON"
echo "配置文件: $CONFIG_FILE"
echo
echo "下一步:"
echo "1. 重启 Claude Desktop"
echo "2. 在 Claude 中尝试说："
echo "   '帮我创建一个 User 模型'"
echo
echo "激活虚拟环境（用于开发）:"
echo "  source activate.sh"
echo
echo "文档:"
echo "  - QUICKSTART.md           - 快速开始"
echo "  - loco-mcp-server/README.md - 完整文档"
echo "  - loco-mcp-server/example_usage.md - 使用示例"
echo
echo "调试:"
echo "  - 查看日志: tail -f $LOG_DIR/mcp*.log"
echo "  - 手动测试: source activate.sh && python3 -m loco_mcp_server.server"
echo "  - 运行命令: make help"
echo
