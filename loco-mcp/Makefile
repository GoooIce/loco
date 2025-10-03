# Loco MCP Server Makefile
# Provides convenient commands for building, testing, and developing loco-mcp

.PHONY: help install install-dev install-rust install-python build test test-rust test-python clean format lint check docs ci benchmark publish

# Default target
help:
	@echo "Loco MCP Server Build System"
	@echo "=========================="
	@echo ""
	@echo "Available targets:"
	@echo "  install        - Install loco-mcp-server for development"
	@echo "  install-dev    - Install development dependencies"
	@echo "  install-rust   - Install Rust toolchain"
	@echo "  install-python - Install Python and dependencies"
	@echo "  build          - Build Rust bindings and Python package"
	@echo "  test           - Run all tests"
	@echo "  test-rust      - Run Rust tests only"
	@echo "  test-python    - Run Python tests only"
	@echo "  test-integration - Run integration tests"
	@echo "  test-performance - Run performance tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  format         - Format code (Rust and Python)"
	@echo "  lint           - Run linting checks"
	@echo "  check          - Run all checks (format, lint, tests)"
	@echo "  docs           - Generate documentation"
	@echo "  benchmark      - Run performance benchmarks"
	@echo "  publish        - Build and publish to PyPI"
	@echo "  ci             - Run CI pipeline locally"
	@echo ""
	@echo "Development workflow:"
	@echo "  make install-dev"
	@echo "  make check"
	@echo "  make test"

# Installation targets
install: install-dev
	@echo "✅ Installation complete!"

install-dev: install-rust install-python
	@echo "Setting up development environment..."
	@if [ ! -d ".venv" ]; then python3 -m venv .venv; fi
	@source .venv/bin/activate && \
		pip install --upgrade pip && \
		pip install maturin[patchelf] black ruff mypy pytest pytest-asyncio pytest-cov && \
		cd loco-bindings && maturin develop --release && \
		cd ../loco-mcp-server && pip install -e .[dev]
	@echo "✅ Development environment setup complete!"

install-rust:
	@echo "Installing Rust toolchain..."
	@if command -v rustup > /dev/null 2>&1; then \
		rustup update; \
		rustup component add rustfmt clippy; \
	else \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		source ~/.cargo/env; \
	fi
	@echo "✅ Rust toolchain installed!"

install-python:
	@echo "Checking Python installation..."
	@python3 -c "import sys; assert sys.version_info >= (3, 11)" || \
		(echo "❌ Python 3.11+ is required" && exit 1)
	@echo "✅ Python $(python3 --version) is available!"

# Build targets
build:
	@echo "Building loco-bindings..."
	@cd loco-bindings && cargo build --release
	@echo "✅ Build complete!"

build-debug:
	@echo "Building loco-bindings in debug mode..."
	@cd loco-bindings && cargo build
	@echo "✅ Debug build complete!"

# Test targets
test: test-rust test-python
	@echo "✅ All tests passed!"

test-rust:
	@echo "Running Rust tests..."
	@cd loco-bindings && cargo test --verbose
	@echo "✅ Rust tests passed!"

test-python:
	@echo "Running Python tests..."
	@cd loco-mcp-server && python -m pytest tests/ -v
	@echo "✅ Python tests passed!"

test-integration:
	@echo "Running integration tests..."
	@cd loco-mcp-server && python -m pytest ../tests/test_integration_real_projects.py -v
	@echo "✅ Integration tests passed!"

test-performance:
	@echo "Running performance tests..."
	@cd loco-mcp-server && python -m pytest ../tests/test_performance_validation.py -v
	@echo "✅ Performance tests passed!"

test-coverage:
	@echo "Running tests with coverage..."
	@cd loco-mcp-server && python -m pytest tests/ --cov=src --cov-report=html --cov-report=term
	@echo "✅ Coverage report generated in loco-mcp-server/htmlcov/"

# Code quality targets
format:
	@echo "Formatting code..."
	@cd loco-bindings && cargo fmt
	@cd loco-mcp-server && ruff format .
	@echo "✅ Code formatted!"

lint:
	@echo "Running linting checks..."
	@cd loco-bindings && cargo clippy -- -D warnings
	@cd loco-mcp-server && ruff check .
	@echo "✅ Linting checks passed!"

type-check:
	@echo "Running type checking..."
	@cd loco-mcp-server && mypy src/
	@echo "✅ Type checking passed!"

check: format lint type-check test
	@echo "✅ All checks passed!"

# Documentation targets
docs:
	@echo "Generating documentation..."
	@cd loco-bindings && cargo doc --no-deps
	@echo "✅ Documentation generated in loco-bindings/target/doc/"

docs-serve:
	@echo "Starting documentation server..."
	@cd loco-bindings && cargo doc --no-deps --open

# Clean targets
clean:
	@echo "Cleaning build artifacts..."
	@cd loco-bindings && cargo clean
	@cd loco-mcp-server && rm -rf build/ dist/ *.egg-info/
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name "*.pyc" -delete 2>/dev/null || true
	@echo "✅ Clean complete!"

clean-all: clean
	@echo "Removing virtual environment..."
	@rm -rf .venv/
	@echo "✅ Deep clean complete!"

# Development targets
dev-setup: clean install-dev
	@echo "✅ Development environment setup complete!"

dev-test: format lint test
	@echo "✅ Development test cycle complete!"

# Benchmark targets
benchmark:
	@echo "Running performance benchmarks..."
	@cd loco-bindings && cargo test --release performance_test -- --nocapture
	@echo "✅ Benchmarks complete!"

# CI/CD targets
ci: format lint test-rust test-python test-integration
	@echo "✅ CI pipeline passed!"

ci-local: ci
	@echo "✅ Local CI check passed!"

# Publishing targets
build-wheel:
	@echo "Building distribution wheels..."
	@cd loco-bindings && maturin build --release --out dist --strip
	@echo "✅ Wheels built in loco-bindings/dist/"

publish-test: build-wheel
	@echo "Publishing to Test PyPI..."
	@cd loco-bindings && pip install twine && \
		twine upload dist/* --repository testpypi
	@echo "✅ Published to Test PyPI!"

publish: build-wheel
	@echo "Publishing to PyPI..."
	@cd loco-bindings && pip install twine && \
		twine upload dist/*
	@echo "✅ Published to PyPI!"

# Utility targets
version:
	@echo "Rust version:"
	@cd loco-bindings && cargo version --version
	@echo "Python version:"
	@cd loco-mcp-server && python -c "import sys; print(f'Python {sys.version}')"

info:
	@echo "Loco MCP Server Information"
	@echo "==========================="
	@echo "Rust version: $$(rustc --version)"
	@echo "Python version: $(python3 --version)"
	@echo "Maturin version: $$(maturin --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo ""
	@echo "Project structure:"
	@echo "loco-bindings/    - Rust bindings library"
	@echo "loco-mcp-server/  - Python MCP server"
	@echo "tests/           - Integration and performance tests"
	@echo "docs/            - Documentation"
	@echo "examples/        - Usage examples"

# Quick start targets
quick-start: install
	@echo "🚀 Quick start complete!"
	@echo ""
	@echo "To start using loco-mcp-server:"
	@echo "1. Activate virtual environment: source .venv/bin/activate"
	@echo "2. Start the server: loco-mcp-server"
	@echo "3. Configure Claude Code to connect to the server"
	@echo ""
	@echo "See README.md for detailed instructions"

# Development shortcuts
t: test
tc: test-coverage
b: build
bd: build-debug
c: clean
f: format
l: lint
d: docs