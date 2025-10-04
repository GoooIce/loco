.PHONY: help setup install test run dev clean lint format check activate

# 默认使用虚拟环境中的 Python
VENV_DIR := $(shell pwd)/.venv
PYTHON := $(VENV_DIR)/bin/python3
PIP := $(VENV_DIR)/bin/pip

help:
	@echo "Loco MCP - 统一项目管理"
	@echo ""
	@echo "环境设置:"
	@echo "  make setup      - 创建虚拟环境并安装所有依赖"
	@echo "  make install    - 完整安装（setup + Claude Desktop 配置）"
	@echo "  make activate   - 显示如何激活虚拟环境"
	@echo ""
	@echo "开发命令:"
	@echo "  make test       - 运行所有测试"
	@echo "  make run        - 运行 MCP 服务器"
	@echo "  make dev        - 开发模式（详细日志）"
	@echo ""
	@echo "代码质量:"
	@echo "  make lint       - 代码检查"
	@echo "  make format     - 代码格式化"
	@echo "  make check      - 完整检查（lint + test）"
	@echo ""
	@echo "清理:"
	@echo "  make clean      - 清理临时文件"
	@echo "  make clean-venv - 删除虚拟环境"
	@echo ""
	@echo "子项目:"
	@echo "  make bindings   - 只构建 loco-bindings"
	@echo "  make server     - 只安装 loco-mcp-server"
	@echo ""

setup:
	@echo "设置虚拟环境..."
	./setup_venv.sh

install:
	@echo "完整安装..."
	./install.sh

activate:
	@echo "要激活虚拟环境，请运行："
	@echo ""
	@echo "  source activate.sh"
	@echo ""
	@echo "或者："
	@echo "  source $(VENV_DIR)/bin/activate"
	@echo ""

bindings:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "构建 loco-bindings..."; \
		cd loco-bindings && $(VENV_DIR)/bin/maturin develop; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

server:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "安装 loco-mcp-server..."; \
		cd loco-mcp-server && $(PIP) install -e .; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

test:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "运行测试..."; \
		cd loco-mcp-server && $(PYTHON) test_server.py; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

run:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "启动 MCP 服务器..."; \
		$(PYTHON) -m loco_mcp_server.server; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

dev:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "启动服务器（开发模式）..."; \
		LOG_LEVEL=DEBUG $(PYTHON) -m loco_mcp_server.server; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

lint:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "运行 linter..."; \
		cd loco-mcp-server && $(VENV_DIR)/bin/ruff check src/ || true; \
		cd loco-mcp-server && $(VENV_DIR)/bin/mypy src/ || true; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

format:
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "格式化代码..."; \
		cd loco-mcp-server && $(VENV_DIR)/bin/black src/; \
		cd loco-mcp-server && $(VENV_DIR)/bin/ruff check --fix src/ || true; \
	else \
		echo "❌ 虚拟环境不存在，请先运行: make setup"; \
		exit 1; \
	fi

check: lint test
	@echo "✅ 所有检查通过"

clean:
	@echo "清理临时文件..."
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	find . -type f -name "*.pyo" -delete 2>/dev/null || true
	find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
	rm -rf loco-mcp-server/build/ loco-mcp-server/dist/ 2>/dev/null || true
	rm -rf loco-bindings/target/wheels/ 2>/dev/null || true
	rm -rf .pytest_cache/ .mypy_cache/ .ruff_cache/ 2>/dev/null || true
	@echo "✅ 清理完成"

clean-venv:
	@echo "删除虚拟环境..."
	rm -rf $(VENV_DIR)
	rm -f activate.sh
	@echo "✅ 虚拟环境已删除"
	@echo "   运行 'make setup' 重新创建"

# 显示当前状态
status:
	@echo "Loco MCP 项目状态"
	@echo "=================================="
	@echo ""
	@if [ -d "$(VENV_DIR)" ]; then \
		echo "✅ 虚拟环境: $(VENV_DIR)"; \
		echo "   Python: $(shell $(PYTHON) --version 2>/dev/null || echo '未找到')"; \
		echo ""; \
		echo "已安装的包:"; \
		$(PIP) list | grep -E "loco-|mcp" 2>/dev/null || echo "   (无相关包)"; \
	else \
		echo "❌ 虚拟环境未设置"; \
		echo "   运行 'make setup' 创建"; \
	fi
	@echo ""
	@if command -v rustc &> /dev/null; then \
		echo "✅ Rust: $$(rustc --version)"; \
	else \
		echo "⚠️  Rust 未安装（loco-bindings 将使用 mock 模式）"; \
	fi
	@echo ""
