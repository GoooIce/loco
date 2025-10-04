# Loco MCP 统一虚拟环境说明

## 🎯 设计理念

使用**一个统一的虚拟环境**来管理整个 `loco-mcp` 项目：

```
loco-mcp/
├── .venv/                    ← 统一虚拟环境
│   ├── bin/
│   │   ├── python3          ← 所有操作使用此 Python
│   │   ├── pip
│   │   └── maturin
│   └── lib/
│       └── python3.12/
│           └── site-packages/
│               ├── loco_bindings/    ← Rust 绑定安装在这里
│               ├── loco_mcp_server/  ← Python 包安装在这里
│               └── mcp/              ← 官方 MCP SDK
├── loco-bindings/           ← 使用 .venv
└── loco-mcp-server/         ← 使用 .venv
```

## 为什么这样设计？

### ✅ 优点

1. **一致性**: 两个子项目使用相同的 Python 环境和依赖
2. **简化**: 不需要管理多个 venv，不会混淆
3. **高效**: 共享依赖，节省磁盘空间
4. **集成**: `loco-bindings` 构建后直接可用于 `loco-mcp-server`

### ❌ 避免的问题

- 🚫 多个虚拟环境导致路径混乱
- 🚫 包版本冲突
- 🚫 `loco-bindings` 安装在错误的 Python 环境
- 🚫 Claude Desktop 使用错误的 Python

## 📋 目录结构

```
loco-mcp/
│
├── .venv/                       # 虚拟环境（gitignored）
│   └── bin/python3              # 所有脚本使用这个
│
├── setup_venv.sh                # 创建虚拟环境 + 安装所有依赖
├── install.sh                   # setup_venv.sh + 配置 Claude Desktop
├── activate.sh                  # 自动生成的激活脚本
├── Makefile                     # 统一命令入口
│
├── loco-bindings/               # 子项目 1: Rust → Python
│   ├── src/lib.rs               # PyO3 绑定
│   ├── pyproject.toml
│   └── (maturin develop 到 ../.venv)
│
└── loco-mcp-server/             # 子项目 2: MCP 服务器
    ├── src/server.py            # Python 服务器
    ├── pyproject.toml
    └── (pip install -e . 到 ../.venv)
```

## 🚀 工作流程

### 首次设置

```bash
cd loco-mcp
./install.sh          # 一次搞定
```

这会：
1. 在 `loco-mcp/.venv` 创建虚拟环境
2. 安装 `maturin`
3. 构建 `loco-bindings` → 安装到 `.venv`
4. 安装 `loco-mcp-server` → 安装到 `.venv`
5. 配置 Claude Desktop 使用 `.venv/bin/python3`

### 日常开发

```bash
# 1. 激活虚拟环境
cd loco-mcp
source activate.sh
# 现在你在 .venv 中

# 2. 开发 Rust 绑定
cd loco-bindings
# 编辑 src/lib.rs
maturin develop        # 重新构建到 .venv

# 3. 开发 Python 服务器
cd ../loco-mcp-server
# 编辑 src/server.py
python3 test_server.py # 直接测试（-e 模式）

# 4. 退出
deactivate
```

### 使用 Makefile（不需要手动激活）

```bash
cd loco-mcp
make run              # 自动使用 .venv/bin/python3
make test             # 自动使用 .venv/bin/python3
make bindings         # 自动使用 .venv/bin/maturin
```

## 🔧 关键配置

### Claude Desktop 配置

**正确**（使用虚拟环境）:
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

**错误**（使用系统 Python）:
```json
{
  "mcpServers": {
    "loco": {
      "command": "python3",  ❌ 找不到 loco_bindings 和 mcp
      "args": ["-m", "loco_mcp_server.server"]
    }
  }
}
```

### Makefile 配置

```makefile
VENV_DIR := $(shell pwd)/.venv
PYTHON := $(VENV_DIR)/bin/python3    # 所有命令使用这个
PIP := $(VENV_DIR)/bin/pip

run:
    $(PYTHON) -m loco_mcp_server.server  # 使用 .venv 中的 Python
```

## 🐛 常见问题

### Q: 如何确认虚拟环境正确？

```bash
cd loco-mcp
source activate.sh

# 检查 Python 路径
which python3
# 应该输出: /Users/.../loco-mcp/.venv/bin/python3

# 检查已安装的包
pip list | grep loco
# 应该看到:
#   loco-bindings    0.1.0
#   loco-mcp-server  0.1.0
```

### Q: loco-bindings 找不到？

```bash
# 1. 确认虚拟环境激活
source activate.sh

# 2. 重新构建
cd loco-bindings
maturin develop

# 3. 验证
python3 -c "import loco_bindings; print('✅ OK')"
```

### Q: Claude Desktop 找不到模块？

检查配置文件中的 `command` 是否使用**绝对路径**到 `.venv/bin/python3`。

不要使用：
- ❌ `python3` (系统 Python)
- ❌ `~/.../.venv/bin/python3` (~ 可能不展开)

使用：
- ✅ `/Users/你的用户名/.../loco-mcp/.venv/bin/python3`

### Q: 如何重置虚拟环境？

```bash
cd loco-mcp
make clean-venv    # 删除 .venv
make setup         # 重新创建并安装
```

## 📊 依赖关系

```
.venv/
├── mcp (官方 SDK)                 ← loco-mcp-server 依赖
├── pyo3 (运行时)                  ← loco-bindings 依赖
├── loco_bindings (Rust 模块)     ← loco-mcp-server 依赖
└── loco_mcp_server (Python 包)   ← Claude Desktop 运行
```

**安装顺序**:
1. 创建 `.venv`
2. 安装基础工具（`pip`, `maturin`）
3. 构建 `loco-bindings` → 安装到 `.venv`
4. 安装 `loco-mcp-server` → 安装到 `.venv`（依赖 `loco-bindings` 和 `mcp`）

## 🎓 最佳实践

### ✅ 推荐

```bash
# 总是从项目根目录开始
cd loco-mcp

# 使用 Makefile（自动处理虚拟环境）
make test
make run

# 或者先激活虚拟环境
source activate.sh
cd loco-bindings
# 进行开发...
```

### ❌ 避免

```bash
# 不要在子项目中单独创建虚拟环境
cd loco-mcp-server
python3 -m venv venv        # ❌ 错误！

# 不要使用系统 Python
cd loco-mcp-server
python3 -m pip install -e . # ❌ 会安装到系统或错误的环境
```

## 📚 相关文件

- `setup_venv.sh` - 虚拟环境设置脚本
- `install.sh` - 完整安装（包含 Claude Desktop 配置）
- `Makefile` - 统一命令入口（自动使用 `.venv`）
- `activate.sh` - 快速激活脚本（自动生成）
- `.gitignore` - 忽略 `.venv/` 和 `activate.sh`

---

**记住**: 一个项目，一个虚拟环境，简单明了！🚀

