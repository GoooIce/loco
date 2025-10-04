# Loco MCP Server 快速开始

## 5 分钟上手

### 1️⃣ 一键安装

```bash
cd loco-mcp
./install.sh
```

这个脚本会自动：
- ✅ 在 `loco-mcp/.venv` 创建统一虚拟环境
- ✅ 安装 Python 依赖（mcp SDK 等）
- ✅ 构建并安装 loco-bindings（Rust → Python）
- ✅ 安装 loco-mcp-server
- ✅ 运行测试验证
- ✅ 自动配置 Claude Desktop

**注意**: 两个子项目（`loco-bindings` 和 `loco-mcp-server`）共享同一个虚拟环境。

### 2️⃣ 配置 Claude Desktop（通常自动完成）

安装脚本会自动配置，如果需要手动配置：

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

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

**关键**: 使用虚拟环境中的 Python（`.venv/bin/python3`），而不是系统 Python。

### 3️⃣ 重启 Claude Desktop

重启后，你应该能看到 🔌 图标，表示 MCP 服务器已连接。

### 4️⃣ 开始使用！

在 Claude 中尝试：

**创建一个用户模型**:
```
帮我创建一个 User 模型，包含 name（string）、email（string）、age（integer）字段
```

**生成博客脚手架**:
```
生成一个 BlogPost 的完整脚手架，使用 API 模式，包含 title 和 content 字段
```

**为现有模型添加控制器**:
```
我有一个 Product 模型，帮我生成 API 控制器
```

## 命令参考

在项目根目录（`loco-mcp/`）使用 Makefile：

```bash
# 查看所有命令
make help

# 初始设置
make setup          # 只创建虚拟环境和安装依赖
make install        # 完整安装（setup + Claude Desktop 配置）

# 开发
make run            # 运行 MCP 服务器
make dev            # 开发模式（详细日志）
make test           # 运行测试

# 虚拟环境
make activate       # 显示如何激活虚拟环境
source activate.sh  # 激活虚拟环境（用于开发）

# 代码质量
make lint           # 代码检查
make format         # 格式化代码
make check          # 完整检查

# 清理
make clean          # 清理临时文件
make clean-venv     # 删除虚拟环境

# 查看状态
make status         # 显示项目状态
```

## 常见问题

### Q: 看不到工具？

**检查**:
1. Claude Desktop 已重启
2. 查看日志: `~/Library/Logs/Claude/mcp*.log`
3. 手动测试: `python3 -m loco_mcp_server.server`

### Q: 工具调用失败？

**确保**:
1. 在 Loco 项目目录中（包含 `Cargo.toml`）
2. `loco-bindings` 已安装: `python3 -c "import loco_bindings"`
3. 项目结构正确（有 `src/models`、`src/controllers` 等）

### Q: 使用 Mock 模式？

如果 `loco-bindings` 未安装（例如没有 Rust 工具链），服务器会使用 mock 模式（仅用于测试）。

要构建真实的绑定：
```bash
# 1. 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 激活虚拟环境并构建
cd loco-mcp
source activate.sh
make bindings       # 或者: cd loco-bindings && maturin develop
```

### Q: 如何开发和调试？

```bash
# 1. 激活虚拟环境
cd loco-mcp
source activate.sh

# 2. 进入子项目
cd loco-bindings     # 或 cd loco-mcp-server

# 3. 开发
# 修改 Rust 代码后重新构建
cd loco-bindings && maturin develop

# 修改 Python 代码后直接测试（已经是 -e 安装）
cd loco-mcp-server && python3 test_server.py
```

## 下一步

- 📖 阅读 [完整文档](loco-mcp-server/README.md)
- 📝 查看 [使用示例](loco-mcp-server/example_usage.md)
- 🔧 了解 [重构细节](REFACTORING_SUMMARY.md)

## 架构

```
Claude Desktop
    ↓ (MCP 协议 - stdio)
loco-mcp-server (Python)
    ↓ (PyO3)
loco-bindings (Rust FFI)
    ↓
loco-gen (Rust)
```

## 支持的操作

| 工具 | 生成内容 |
|-----|---------|
| `loco_generate_model` | 模型 + 迁移 |
| `loco_generate_scaffold` | 模型 + 控制器 + 视图 + 迁移 |
| `loco_generate_controller_view` | 控制器 + 视图 |

## 脚手架类型

- **api** - REST API（JSON）
- **html** - 服务器渲染 HTML
- **htmx** - HTMX 交互式界面

Happy coding! 🚀

