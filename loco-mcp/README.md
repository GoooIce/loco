# Loco MCP - Model Context Protocol 服务器

这是 [Loco](https://loco.rs) Web 框架的 **MCP (Model Context Protocol) 服务器**实现，允许 AI 助手（如 Claude）直接生成 Loco 项目的代码。

## 📦 项目结构

```
loco-mcp/
├── .venv/                    # 统一虚拟环境（两个子项目共享）
├── loco-bindings/            # Rust → Python FFI 绑定
│   ├── src/
│   │   ├── lib.rs           # PyO3 绑定实现
│   │   └── error.rs         # 错误处理
│   └── pyproject.toml
├── loco-mcp-server/          # MCP 服务器（Python）
│   ├── src/
│   │   ├── server.py        # 主服务器（使用官方 mcp SDK）
│   │   ├── tools.py         # 工具实现
│   │   └── ...
│   └── pyproject.toml
├── setup_venv.sh             # 虚拟环境设置脚本
├── install.sh                # 完整安装脚本
├── Makefile                  # 统一命令入口
└── README.md                 # 本文件
```

## 🚀 快速开始

### 前置要求

- **Python 3.11+**
- **Rust 工具链**（可选，用于构建真实的绑定；没有则使用 mock 模式）
- **Claude Desktop** 或其他 MCP 客户端

### 安装

#### 方法 1: 一键安装（推荐）

```bash
cd loco-mcp
./install.sh
```

这会自动完成所有设置，包括配置 Claude Desktop。

#### 方法 2: 分步安装

```bash
# 1. 设置虚拟环境
cd loco-mcp
./setup_venv.sh

# 2. 激活虚拟环境
source activate.sh

# 3. 手动配置 Claude Desktop（见下文）
```

#### 方法 3: 使用 Makefile

```bash
cd loco-mcp
make install    # 完整安装
# 或
make setup      # 只设置虚拟环境
```

### 配置 Claude Desktop

编辑配置文件：
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

添加以下内容（**使用你的实际路径**）：

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

**重要**: 必须使用虚拟环境中的 Python 路径（`.venv/bin/python3`）！

## 🛠️ 使用方式

### 在 Claude 中使用

重启 Claude Desktop 后，你可以直接说：

```
帮我创建一个 User 模型，包含 name（string）、email（string）、age（integer）字段
```

```
生成一个 BlogPost 的完整脚手架，使用 API 模式
```

```
为现有的 Product 模型生成 HTML 控制器
```

详细示例请查看 [example_usage.md](loco-mcp-server/example_usage.md)。

### 命令行使用

```bash
# 查看所有命令
make help

# 运行服务器（手动测试）
make run

# 开发模式（详细日志）
make dev

# 运行测试
make test

# 查看项目状态
make status
```

## 🏗️ 架构

```
┌─────────────────────┐
│   Claude Desktop    │  任何 MCP 客户端
│    (MCP Client)     │
└──────────┬──────────┘
           │
           │ MCP Protocol (stdio)
           │ https://modelcontextprotocol.io
           │
┌──────────▼──────────┐
│  loco-mcp-server    │
│   (Python 包)       │  使用官方 mcp SDK
│                     │  src/server.py
└──────────┬──────────┘
           │
           │ PyO3 FFI
           │
┌──────────▼──────────┐
│   loco-bindings     │
│  (Rust → Python)    │  PyO3 绑定
│                     │  src/lib.rs
└──────────┬──────────┘
           │
           │ 调用
           │
┌──────────▼──────────┐
│     loco-gen        │
│   (Rust 代码生成)   │  Loco 框架的生成器
└─────────────────────┘
```

## 🔧 开发

### 激活虚拟环境

```bash
cd loco-mcp
source activate.sh
```

### 修改代码

**修改 Rust 代码（loco-bindings）**:
```bash
cd loco-bindings
# 编辑 src/lib.rs
maturin develop    # 重新构建
```

**修改 Python 代码（loco-mcp-server）**:
```bash
cd loco-mcp-server
# 编辑 src/server.py 或 src/tools.py
# 无需重新安装（已经是 -e 模式）
python3 test_server.py    # 测试
```

### 调试

```bash
# 1. 查看 Claude Desktop 日志
tail -f ~/Library/Logs/Claude/mcp*.log

# 2. 手动运行服务器
source activate.sh
python3 -m loco_mcp_server.server

# 3. 启用详细日志
LOG_LEVEL=DEBUG python3 -m loco_mcp_server.server

# 4. 运行测试
make test
```

## 📚 文档

- [QUICKSTART.md](QUICKSTART.md) - 5 分钟快速开始
- [loco-mcp-server/README.md](loco-mcp-server/README.md) - 服务器详细文档
- [loco-mcp-server/example_usage.md](loco-mcp-server/example_usage.md) - 使用示例
- [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md) - 重构说明

## 🎯 提供的工具

| 工具名称 | 功能 | 生成内容 |
|---------|------|---------|
| `loco_generate_model` | 生成模型 | 模型文件 + 数据库迁移 |
| `loco_generate_scaffold` | 生成脚手架 | 模型 + 控制器 + 视图 + 迁移 |
| `loco_generate_controller_view` | 生成控制器和视图 | 控制器 + 视图模板 |

### 脚手架类型

- **api** - REST API（返回 JSON）
- **html** - 服务器渲染 HTML
- **htmx** - HTMX 交互式界面

## 🧪 测试

```bash
# 运行测试套件
make test

# 或者手动
source activate.sh
cd loco-mcp-server
python3 test_server.py
```

## 🐛 常见问题

### Q: Claude Desktop 看不到工具？

**检查**:
1. Claude Desktop 已重启
2. 配置文件路径正确
3. 使用虚拟环境的 Python 路径（不是系统 Python）
4. 查看日志: `tail -f ~/Library/Logs/Claude/mcp*.log`

### Q: 工具调用失败？

**确保**:
1. 在 Loco 项目目录中运行（包含 `Cargo.toml`）
2. 项目结构正确（有 `src/models/`、`src/controllers/` 等）
3. `loco-bindings` 已正确安装: `python3 -c "import loco_bindings"`

### Q: 如何更新？

```bash
cd loco-mcp
source activate.sh

# 更新 loco-bindings
cd loco-bindings && maturin develop

# 更新 loco-mcp-server
cd ../loco-mcp-server && pip install -e .
```

### Q: 虚拟环境在哪里？

统一的虚拟环境位于 `loco-mcp/.venv/`，由两个子项目共享：
- `loco-bindings` 使用它来构建 Rust 绑定
- `loco-mcp-server` 使用它来运行 Python 服务器

激活方式：
```bash
cd loco-mcp
source activate.sh
# 或
source .venv/bin/activate
```

### Q: 没有 Rust 工具链怎么办？

服务器会使用 mock 模式（返回模拟结果）。要安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
cd loco-mcp
make bindings    # 重新构建
```

## 🔄 清理

```bash
# 清理临时文件
make clean

# 删除虚拟环境（重新开始）
make clean-venv
make setup    # 重新创建
```

## 📖 标准和协议

本项目实现了 [Model Context Protocol (MCP)](https://modelcontextprotocol.io) 规范：
- 使用官方 [mcp Python SDK](https://github.com/modelcontextprotocol/python-sdk)
- 通过 stdio 通信
- 兼容所有 MCP 客户端

## 🤝 贡献

欢迎贡献！请查看各子项目的 README 了解详情。

## 📄 许可证

MIT OR Apache-2.0

---

**Happy coding with Loco + AI!** 🚀

