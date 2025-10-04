# 虚拟环境迁移指南

## 背景

之前可能存在多个虚拟环境，导致包安装路径混乱。现在统一使用一个虚拟环境。

## 新架构

```
loco-mcp/
├── .venv/                        ← 唯一的虚拟环境
│   ├── bin/
│   │   ├── python3              ← 所有操作使用这个
│   │   ├── pip
│   │   └── maturin
│   └── lib/
│       └── python3.12/
│           └── site-packages/
│               ├── loco_bindings/      ← 从 loco-bindings/ 构建
│               ├── loco_mcp_server/    ← 从 loco-mcp-server/ 构建
│               └── mcp/                ← 官方 SDK
├── loco-bindings/                ← 不再有自己的 venv
│   └── (maturin develop → ../venv)
└── loco-mcp-server/              ← 不再有自己的 venv
    └── (pip install -e . → ../.venv)
```

## 迁移步骤

### 1. 清理旧环境（可选）

如果你之前有多个虚拟环境：

```bash
cd loco-mcp

# 列出可能的旧虚拟环境
find . -name "venv" -type d
find . -name ".venv" -type d

# 删除子项目中的虚拟环境
rm -rf loco-bindings/venv
rm -rf loco-bindings/.venv
rm -rf loco-mcp-server/venv
rm -rf loco-mcp-server/.venv

# 只保留根目录的 .venv（如果要重新开始，也可以删除）
# rm -rf .venv
```

### 2. 重新设置（推荐）

```bash
cd loco-mcp

# 方法 A: 完整安装（推荐）
./install.sh

# 方法 B: 只设置虚拟环境
./setup_venv.sh

# 方法 C: 使用 Makefile
make install    # 或 make setup
```

### 3. 验证

```bash
cd loco-mcp
source activate.sh

# 检查 Python 路径
which python3
# 应该输出: /Users/.../loco-mcp/.venv/bin/python3

# 检查已安装的包
pip list | grep -E "loco-|mcp"
# 应该看到:
#   loco-bindings       0.1.0
#   loco-mcp-server     0.1.0
#   mcp                 1.x.x

# 测试导入
python3 << 'EOF'
import loco_bindings
import loco_mcp_server
import mcp
print("✅ 所有包都可用")
EOF
```

### 4. 更新 Claude Desktop 配置

**重要**: 确保使用新的 Python 路径！

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`:

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

重启 Claude Desktop。

## 对比

### ❌ 之前（混乱）

```
loco-mcp/
├── loco-bindings/
│   ├── venv/                     ← 第 1 个虚拟环境
│   │   └── lib/.../loco_bindings/
│   └── ...
├── loco-mcp-server/
│   ├── venv/                     ← 第 2 个虚拟环境
│   │   └── lib/.../loco_mcp_server/
│   └── ...
└── .venv/                        ← 第 3 个虚拟环境？

问题:
- loco-bindings 可能安装在错误的环境
- loco-mcp-server 找不到 loco-bindings
- Claude Desktop 使用哪个 Python？
```

### ✅ 现在（清晰）

```
loco-mcp/
├── .venv/                        ← 唯一虚拟环境
│   └── lib/python3.12/site-packages/
│       ├── loco_bindings/        ← 从 loco-bindings 构建
│       ├── loco_mcp_server/      ← 从 loco-mcp-server 安装
│       └── mcp/                  ← 官方 SDK
├── loco-bindings/
│   └── (无 venv，使用 ../.venv)
└── loco-mcp-server/
    └── (无 venv，使用 ../.venv)

优点:
✅ 一个虚拟环境，路径明确
✅ loco_bindings 总是可用
✅ Claude Desktop 配置简单
✅ 开发调试方便
```

## 常见问题

### Q: 我已经有 .venv，需要删除吗？

**不需要**。运行 `./install.sh` 时会询问是否重新创建。

如果想从头开始：
```bash
make clean-venv
make setup
```

### Q: 子项目还能独立工作吗？

**可以**，只要先激活虚拟环境：

```bash
cd loco-mcp
source activate.sh

# 现在可以在子项目中工作
cd loco-bindings
maturin develop        # 使用 ../.venv

cd ../loco-mcp-server
python3 test_server.py # 使用 ../.venv
```

### Q: 我能在子项目中运行 `python3 -m venv venv` 吗？

**不推荐**。这会创建新的虚拟环境，导致混乱。

始终使用根目录的 `.venv`：
```bash
cd loco-mcp
source activate.sh
# 然后再进入子目录工作
```

### Q: Makefile 如何处理虚拟环境？

Makefile 自动使用 `.venv/bin/python3`，无需手动激活：

```makefile
VENV_DIR := $(shell pwd)/.venv
PYTHON := $(VENV_DIR)/bin/python3

run:
    $(PYTHON) -m loco_mcp_server.server
```

你可以直接运行 `make run`，无需 `source activate.sh`。

## 工作流对比

### ❌ 之前

```bash
# 不清楚该激活哪个虚拟环境
cd loco-mcp/loco-bindings
source venv/bin/activate        # 还是这个？
maturin develop

cd ../loco-mcp-server
source venv/bin/activate        # 还是这个？
python3 test_server.py

# loco-mcp-server 找不到 loco-bindings！
# 因为它们在不同的虚拟环境中
```

### ✅ 现在

```bash
# 方法 1: 手动激活（推荐用于开发）
cd loco-mcp
source activate.sh              # 只需一次

cd loco-bindings
maturin develop                 # 安装到 ../.venv

cd ../loco-mcp-server
python3 test_server.py          # 能找到 loco-bindings！

deactivate                      # 完成后退出

# 方法 2: 使用 Makefile（无需激活）
cd loco-mcp
make test                       # 自动使用 .venv
make run                        # 自动使用 .venv
```

## 新的最佳实践

### ✅ 推荐

```bash
# 1. 所有操作从项目根目录开始
cd loco-mcp

# 2. 使用提供的脚本
./install.sh                    # 完整安装
source activate.sh              # 激活开发环境
make help                       # 查看命令

# 3. 开发时激活虚拟环境
source activate.sh
cd loco-bindings
# 进行开发...
```

### ❌ 避免

```bash
# 不要在子项目中创建虚拟环境
cd loco-mcp-server
python3 -m venv venv            # ❌ 不要这样做！

# 不要使用系统 Python 安装
pip install -e .                # ❌ 会安装到系统
```

## 文件清单

新增和更新的文件：

```
loco-mcp/
├── .gitignore                  # 新增：忽略 .venv/, activate.sh
├── setup_venv.sh               # 新增：虚拟环境设置
├── install.sh                  # 更新：完整安装脚本
├── activate.sh                 # 自动生成：快速激活
├── Makefile                    # 新增：统一命令
├── README.md                   # 更新：新架构说明
├── QUICKSTART.md               # 更新：新工作流
├── VENV_SETUP.md               # 新增：详细说明
├── VENV_MIGRATION.md           # 本文件：迁移指南
└── CHEATSHEET.md               # 新增：命令速查
```

## 总结

| 方面 | 之前 | 现在 |
|-----|------|------|
| 虚拟环境数量 | 2-3 个 | 1 个 |
| 包安装位置 | 分散 | 统一在 `.venv` |
| 开发流程 | 复杂 | 简单清晰 |
| Claude Desktop 配置 | 容易出错 | 明确路径 |
| Makefile 支持 | 无 | 完整支持 |

**建议**: 如果遇到任何导入或路径问题，直接运行：
```bash
cd loco-mcp
make clean-venv
make install
```

这会重新创建一个干净的环境。

---

**欢迎使用新的统一虚拟环境架构！** 🎉

