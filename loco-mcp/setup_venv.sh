#!/bin/bash
# Loco MCP 统一虚拟环境设置脚本

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$PROJECT_ROOT/.venv"

echo "=================================="
echo "Loco MCP 统一虚拟环境设置"
echo "=================================="
echo
echo "项目根目录: $PROJECT_ROOT"
echo "虚拟环境位置: $VENV_DIR"
echo

# 检查 Python 版本
echo "检查 Python 版本..."
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
    PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d. -f1)
    PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d. -f2)
    
    echo "✅ 找到 Python $PYTHON_VERSION"
    
    if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 11 ]); then
        echo "❌ Python 版本过低"
        echo "   需要 Python 3.11 或更高版本"
        exit 1
    fi
else
    echo "❌ 未找到 Python 3"
    echo "   请先安装 Python 3.11 或更高版本"
    exit 1
fi

# 创建虚拟环境
echo
echo "步骤 1: 创建虚拟环境..."
if [ -d "$VENV_DIR" ]; then
    echo "⚠️  虚拟环境已存在"
    read -p "是否删除并重新创建？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "删除旧环境..."
        rm -rf "$VENV_DIR"
        python3 -m venv "$VENV_DIR"
        echo "✅ 虚拟环境已重新创建"
    else
        echo "ℹ️  保留现有虚拟环境"
    fi
else
    python3 -m venv "$VENV_DIR"
    echo "✅ 虚拟环境创建成功"
fi

# 激活虚拟环境
echo
echo "步骤 2: 激活虚拟环境..."
source "$VENV_DIR/bin/activate"
echo "✅ 虚拟环境已激活"

# 升级 pip
echo
echo "步骤 3: 升级 pip..."
pip install --upgrade pip setuptools wheel
echo "✅ pip 已升级"

# 安装 maturin（用于构建 Rust 绑定）
echo
echo "步骤 4: 安装构建工具..."
pip install maturin
echo "✅ maturin 已安装"

# 安装 loco-bindings
echo
echo "步骤 5: 构建并安装 loco-bindings..."
cd "$PROJECT_ROOT/loco-bindings"
if maturin develop; then
    echo "✅ loco-bindings 已安装"
else
    echo "⚠️  loco-bindings 安装失败（可能缺少 Rust 工具链）"
    echo "   服务器将使用 mock 模式运行"
    echo "   要构建真实的绑定，请先安装 Rust："
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

# 安装 loco-mcp-server
echo
echo "步骤 6: 安装 loco-mcp-server..."
cd "$PROJECT_ROOT/loco-mcp-server"
pip install -e .
echo "✅ loco-mcp-server 已安装"

# 验证安装
echo
echo "步骤 7: 验证安装..."
echo "检查已安装的包："
pip list | grep -E "loco-|mcp"

echo
echo "测试导入..."
python3 -c "import loco_mcp_server; print('✅ loco_mcp_server 可用')" || echo "❌ loco_mcp_server 导入失败"
python3 -c "import loco_bindings; print('✅ loco_bindings 可用')" 2>/dev/null || echo "⚠️  loco_bindings 不可用（将使用 mock 模式）"
python3 -c "import mcp; print('✅ mcp SDK 可用')" || echo "❌ mcp SDK 导入失败"

# 运行测试
echo
echo "步骤 8: 运行测试..."
cd "$PROJECT_ROOT/loco-mcp-server"
python3 test_server.py || {
    echo "⚠️  测试失败，但安装已完成"
}

# 创建激活脚本
echo
echo "步骤 9: 创建便捷激活脚本..."
cat > "$PROJECT_ROOT/activate.sh" <<'EOF'
#!/bin/bash
# 快速激活 Loco MCP 虚拟环境

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$PROJECT_ROOT/.venv/bin/activate"
echo "✅ Loco MCP 虚拟环境已激活"
echo "   Python: $(which python3)"
echo "   pip: $(which pip)"
echo ""
echo "快速命令:"
echo "  deactivate          - 退出虚拟环境"
echo "  cd loco-bindings    - 进入 loco-bindings 目录"
echo "  cd loco-mcp-server  - 进入 loco-mcp-server 目录"
echo "  make help           - 查看可用命令"
EOF
chmod +x "$PROJECT_ROOT/activate.sh"
echo "✅ 创建了 activate.sh"

# 完成
echo
echo "=================================="
echo "🎉 虚拟环境设置完成！"
echo "=================================="
echo
echo "虚拟环境位置: $VENV_DIR"
echo
echo "激活虚拟环境:"
echo "  source $PROJECT_ROOT/activate.sh"
echo "  # 或者"
echo "  source $VENV_DIR/bin/activate"
echo
echo "退出虚拟环境:"
echo "  deactivate"
echo
echo "下一步:"
echo "  1. source activate.sh"
echo "  2. cd loco-mcp-server"
echo "  3. ./install.sh  (配置 Claude Desktop)"
echo
echo "或者直接运行顶层的安装:"
echo "  ./install.sh"
echo

