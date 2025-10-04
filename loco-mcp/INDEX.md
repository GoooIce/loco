# Loco MCP 文档索引

欢迎！这里是所有文档的快速导航。

## 🚀 快速开始

刚接触项目？从这里开始：

1. **[QUICKSTART.md](QUICKSTART.md)** - 5 分钟快速开始指南
2. **[SETUP_COMPLETE.md](SETUP_COMPLETE.md)** - 设置完成后的验证和下一步
3. **[CHEATSHEET.md](CHEATSHEET.md)** - 常用命令速查表

```bash
# 一键安装
./install.sh
```

## 📖 主要文档

### 核心文档

| 文档 | 适合 | 内容 |
|------|------|------|
| **[README.md](README.md)** | 所有人 | 项目总览、架构、快速开始 |
| **[QUICKSTART.md](QUICKSTART.md)** | 新用户 | 5 分钟快速上手指南 |
| **[CHEATSHEET.md](CHEATSHEET.md)** | 开发者 | 命令速查表 |

### 虚拟环境

| 文档 | 适合 | 内容 |
|------|------|------|
| **[VENV_SETUP.md](VENV_SETUP.md)** | 开发者 | 统一虚拟环境详细说明 |
| **[VENV_MIGRATION.md](VENV_MIGRATION.md)** | 维护者 | 从旧架构迁移到新架构 |
| **[SETUP_COMPLETE.md](SETUP_COMPLETE.md)** | 新用户 | 设置完成后的验证清单 |

### 技术文档

| 文档 | 适合 | 内容 |
|------|------|------|
| **[REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md)** | 开发者 | MCP Server 重构说明 |
| **[docs/BINDINGS_REFACTORING.md](docs/BINDINGS_REFACTORING.md)** | 开发者 | Rust Bindings 重构说明 |
| **[loco-mcp-server/README.md](loco-mcp-server/README.md)** | 开发者 | 服务器实现细节 |
| **[loco-mcp-server/example_usage.md](loco-mcp-server/example_usage.md)** | 用户 | 10+ 实际使用场景 |

## 🎯 按场景查找

### 我想...安装和配置

- 🆕 首次安装 → [QUICKSTART.md](QUICKSTART.md)
- 🔄 从旧版本迁移 → [VENV_MIGRATION.md](VENV_MIGRATION.md)
- ✅ 验证安装 → [SETUP_COMPLETE.md](SETUP_COMPLETE.md)
- ⚙️ 配置 Claude Desktop → [README.md#配置-claude-desktop](README.md)

### 我想...开发和调试

- 📝 日常开发流程 → [VENV_SETUP.md#工作流程](VENV_SETUP.md)
- 🐛 调试问题 → [CHEATSHEET.md#调试](CHEATSHEET.md)
- 🔧 修改 Rust 代码 → [CHEATSHEET.md#修改-rust-代码](CHEATSHEET.md)
- 🐍 修改 Python 代码 → [CHEATSHEET.md#修改-python-代码](CHEATSHEET.md)

### 我想...使用和学习

- 💬 在 Claude 中使用 → [loco-mcp-server/example_usage.md](loco-mcp-server/example_usage.md)
- 📚 了解工具 → [README.md#提供的工具](README.md)
- 🏗️ 理解架构 → [README.md#架构](README.md)
- 🔄 了解重构 → [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md)

### 我遇到...问题

- ❓ 常见问题 → [QUICKSTART.md#常见问题](QUICKSTART.md)
- 🔍 故障排查 → [CHEATSHEET.md#故障排查](CHEATSHEET.md)
- 📊 检查状态 → `make status`

## 📂 文件结构

```
loco-mcp/
├── 📄 文档（你在这里）
│   ├── INDEX.md                   # 本文件：文档索引
│   ├── README.md                  # 项目主页
│   ├── QUICKSTART.md              # 快速开始
│   ├── CHEATSHEET.md              # 命令速查
│   ├── VENV_SETUP.md              # 虚拟环境详解
│   ├── VENV_MIGRATION.md          # 迁移指南
│   ├── SETUP_COMPLETE.md          # 设置完成
│   └── REFACTORING_SUMMARY.md     # MCP Server 重构
│
├── 📚 详细文档
│   └── docs/
│       ├── BINDINGS_REFACTORING.md  # Bindings 重构说明
│       ├── API.md                   # API 文档
│       └── CLAUDE_CODE_SETUP.md     # Claude Code 配置
│
├── 🔧 脚本和工具
│   ├── install.sh                 # 完整安装
│   ├── setup_venv.sh              # 虚拟环境设置
│   ├── Makefile                   # 命令入口
│   └── .gitignore                 # Git 忽略规则
│
├── 🐍 Python 虚拟环境
│   └── .venv/                     # 统一虚拟环境
│
├── 🦀 Rust 绑定
│   └── loco-bindings/
│       ├── README.md              # Rust 绑定文档
│       └── src/
│
└── 🐍 MCP 服务器
    └── loco-mcp-server/
        ├── README.md              # 服务器文档
        ├── example_usage.md       # 使用示例
        └── src/
```

## 🎓 学习路径

### 路径 1: 快速使用（5 分钟）

1. [QUICKSTART.md](QUICKSTART.md) - 安装和配置
2. [loco-mcp-server/example_usage.md](loco-mcp-server/example_usage.md) - 使用示例
3. 开始在 Claude 中使用！

### 路径 2: 深入理解（30 分钟）

1. [README.md](README.md) - 项目概览
2. [VENV_SETUP.md](VENV_SETUP.md) - 虚拟环境架构
3. [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md) - MCP Server 重构
4. [docs/BINDINGS_REFACTORING.md](docs/BINDINGS_REFACTORING.md) - Rust Bindings 重构
5. [loco-mcp-server/README.md](loco-mcp-server/README.md) - 服务器实现

### 路径 3: 开发贡献（1 小时）

1. [VENV_SETUP.md](VENV_SETUP.md) - 开发环境
2. [loco-mcp-server/README.md](loco-mcp-server/README.md) - 服务器架构
3. [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md) - MCP Server 重构
4. [docs/BINDINGS_REFACTORING.md](docs/BINDINGS_REFACTORING.md) - Bindings 重构
5. 阅读源码：`loco-mcp-server/src/` 和 `loco-bindings/src/`

## 🔗 外部资源

- [Model Context Protocol](https://modelcontextprotocol.io) - 官方规范
- [MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk) - Python SDK
- [Loco Framework](https://loco.rs) - Loco Web 框架
- [PyO3](https://pyo3.rs) - Rust ↔ Python 绑定

## 💡 快速提示

### 刚开始？

```bash
./install.sh              # 一键安装
# 重启 Claude Desktop
# 开始对话！
```

### 需要帮助？

```bash
make help                 # 查看所有命令
make status               # 查看当前状态
```

### 遇到问题？

1. 查看 [CHEATSHEET.md#故障排查](CHEATSHEET.md)
2. 运行 `make clean-venv && make setup`
3. 查看日志：`tail -f ~/Library/Logs/Claude/mcp*.log`

## 📧 获取帮助

1. 查看 [常见问题](QUICKSTART.md#常见问题)
2. 查看 [故障排查](CHEATSHEET.md#故障排查)
3. 运行 `make status` 检查状态
4. 查看服务器日志

## 🎉 快速参考

```bash
# 安装
./install.sh

# 命令
make help               # 帮助
make test               # 测试
make run                # 运行服务器
make status             # 查看状态

# 开发
source activate.sh      # 激活虚拟环境
make bindings           # 构建 Rust 绑定
deactivate              # 退出虚拟环境

# 清理
make clean              # 清理临时文件
make clean-venv         # 删除虚拟环境
```

---

**找不到你需要的？** 查看 [README.md](README.md) 或运行 `make help`。

**开始使用**: [QUICKSTART.md](QUICKSTART.md) → 5 分钟上手！

