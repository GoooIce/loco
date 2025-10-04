# Loco MCP 命令速查表

## 🚀 快速开始

```bash
cd loco-mcp
./install.sh                    # 一键安装所有内容
# 重启 Claude Desktop
```

## 📦 虚拟环境

```bash
# 位置
loco-mcp/.venv/                 # 统一虚拟环境

# 激活
source activate.sh              # 快速激活
source .venv/bin/activate       # 标准方式

# 退出
deactivate

# 检查
which python3                   # 应显示 .venv/bin/python3
pip list | grep loco            # 查看已安装的 loco 包
```

## 🛠️ 常用命令（使用 Makefile）

```bash
# 📋 帮助
make help                       # 显示所有命令

# 🔧 设置
make setup                      # 创建 venv + 安装依赖
make install                    # setup + 配置 Claude Desktop

# 🏃 运行
make run                        # 运行 MCP 服务器
make dev                        # 开发模式（详细日志）
make test                       # 运行测试

# 📊 状态
make status                     # 显示项目状态

# 🧹 清理
make clean                      # 清理临时文件
make clean-venv                 # 删除虚拟环境
```

## 🔨 开发工作流

### 修改 Rust 代码（loco-bindings）

```bash
cd loco-mcp
source activate.sh
cd loco-bindings

# 编辑 src/lib.rs
vim src/lib.rs

# 重新构建
maturin develop

# 测试
python3 -c "import loco_bindings; print('OK')"
```

### 修改 Python 代码（loco-mcp-server）

```bash
cd loco-mcp
source activate.sh
cd loco-mcp-server

# 编辑 src/server.py
vim src/server.py

# 无需重新安装（-e 模式）
python3 test_server.py
```

## 🐛 调试

```bash
# 查看日志
tail -f ~/Library/Logs/Claude/mcp*.log    # macOS
tail -f ~/.cache/Claude/logs/mcp*.log     # Linux

# 手动运行服务器
source activate.sh
python3 -m loco_mcp_server.server

# 详细日志
LOG_LEVEL=DEBUG python3 -m loco_mcp_server.server

# 测试导入
python3 -c "import loco_mcp_server; print('server OK')"
python3 -c "import loco_bindings; print('bindings OK')"
python3 -c "import mcp; print('mcp SDK OK')"
```

## 📝 Claude Desktop 配置

**配置文件位置**:
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**配置内容**（使用你的实际路径）:
```json
{
  "mcpServers": {
    "loco": {
      "command": "/Users/你的用户名/Code/framework/loco/loco-mcp/.venv/bin/python3",
      "args": ["-m", "loco_mcp_server.server"]
    }
  }
}
```

**验证**: 重启 Claude Desktop，看到 🔌 图标即成功。

## 💬 在 Claude 中使用

```
# 创建模型
帮我创建一个 User 模型，包含 name、email、age 字段

# 生成脚手架
生成一个 BlogPost 的完整 API 脚手架

# 生成控制器
为现有的 Product 模型生成 HTML 控制器
```

## 🆘 故障排查

### 问题: 看不到工具

```bash
# 1. 检查配置
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# 2. 验证 Python 路径
ls -la /path/to/loco-mcp/.venv/bin/python3

# 3. 查看日志
tail -f ~/Library/Logs/Claude/mcp*.log

# 4. 重启 Claude Desktop
```

### 问题: 工具调用失败

```bash
# 1. 确认在 Loco 项目中
ls Cargo.toml           # 应该存在

# 2. 检查 loco-bindings
source activate.sh
python3 -c "import loco_bindings"

# 3. 重新构建
cd loco-bindings
maturin develop
```

### 问题: 模块导入错误

```bash
# 1. 检查虚拟环境
source activate.sh
which python3           # 应该是 .venv/bin/python3

# 2. 重新安装
cd loco-mcp
make clean-venv
make setup
```

## 📚 文档

| 文档 | 内容 |
|------|------|
| `README.md` | 项目总览 |
| `QUICKSTART.md` | 5 分钟快速开始 |
| `VENV_SETUP.md` | 虚拟环境详细说明 |
| `REFACTORING_SUMMARY.md` | 重构说明 |
| `loco-mcp-server/README.md` | 服务器文档 |
| `loco-mcp-server/example_usage.md` | 使用示例 |

## 🔑 关键路径

```
loco-mcp/
├── .venv/bin/python3              # Claude Desktop 使用的 Python
├── setup_venv.sh                  # 虚拟环境设置
├── install.sh                     # 完整安装
├── Makefile                       # 命令入口
└── activate.sh                    # 激活脚本（自动生成）
```

## 🎯 提供的 MCP 工具

| 工具 | 功能 |
|------|------|
| `loco_generate_model` | 模型 + 迁移 |
| `loco_generate_scaffold` | 完整脚手架（模型+控制器+视图） |
| `loco_generate_controller_view` | 控制器 + 视图 |

## 🔗 有用的链接

- [Model Context Protocol](https://modelcontextprotocol.io)
- [Loco Framework](https://loco.rs)
- [PyO3](https://pyo3.rs)

---

**提示**: 遇到问题？运行 `make status` 查看当前状态。

