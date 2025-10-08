# Loco MCP Server

一个基于 **Model Context Protocol (MCP)** 的服务器，为 Claude 和其他 AI 助手提供 Loco-rs 框架的代码生成工具。

## 功能特性

该 MCP 服务器通过 Python 绑定暴露了 loco-gen 的核心功能以及 CLI 操作工具：

### 代码生成工具
- **`loco_generate_model`** - 生成 Loco 模型和数据库迁移文件
- **`loco_generate_scaffold`** - 生成完整的 CRUD 脚手架（模型 + 控制器 + 视图）
- **`loco_generate_controller_view`** - 为现有模型生成控制器和视图

### CLI 操作工具
- **`migrate_db`** - 执行数据库迁移操作，支持审批验证和审计日志
- **`rotate_keys`** - 轮换服务账号密钥，包含安全验证和合规检查
- **`clean_temp`** - 清理临时文件和目录，具备安全检查机制

## 安装

### 前置要求

- Python 3.11+
- Rust (用于构建 loco-bindings)
- 一个 Loco-rs 项目
- 自动化服务账号配置（用于 CLI 操作）
- 审计日志目录权限：`/var/log/loco-mcp/`

### 安装步骤

1. 安装依赖：

```bash
cd loco-mcp/loco-mcp-server
pip install -e .
```

2. 构建 loco-bindings（如果还没有构建）：

```bash
cd ../loco-bindings
maturin develop
```

## 使用方法

### 在 Claude Desktop 中配置

在 Claude Desktop 的配置文件中添加：

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "loco": {
      "command": "python",
      "args": ["-m", "loco_mcp_server.server"],
      "env": {
        "PYTHONPATH": "/path/to/loco-mcp/loco-mcp-server/src"
      }
    }
  }
}
```

或者使用 `uv` 运行：

```json
{
  "mcpServers": {
    "loco": {
      "command": "uv",
      "args": ["run", "--directory", "/path/to/loco-mcp/loco-mcp-server", "python", "-m", "loco_mcp_server.server"],
      "env": {
        "LOCO_MCP_AUDIT_LOG_PATH": "/var/log/loco-mcp/audit.log",
        "LOCO_MCP_DEFAULT_TIMEOUT": "60",
        "LOCO_MCP_ENVIRONMENT": "development"
      }
    }
  }
}
```

### 工具使用示例

在 Claude 中，你可以这样请求：

#### CLI 操作工具示例

##### 数据库迁移
```
执行数据库迁移，需要 ops_lead 和 security_officer 的审批
```

Claude 会调用：
```json
{
  "tool": "migrate_db",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "staging",
    "approvals": ["ops_lead", "security_officer"],
    "timeout_seconds": 60,
    "dependencies": ["postgres", "redis"]
  }
}
```

##### 密钥轮换
```
轮换服务账号密钥，需要 security_officer 和 cto 的审批
```

Claude 会调用：
```json
{
  "tool": "rotate_keys",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "production",
    "approvals": ["security_officer", "cto"],
    "timeout_seconds": 300,
    "dependencies": ["kms"]
  }
}
```

##### 临时文件清理
```
清理临时文件，需要 ops_lead 的审批
```

Claude 会调用：
```json
{
  "tool": "clean_temp",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "development",
    "approvals": ["ops_lead"],
    "timeout_seconds": 60,
    "dependencies": ["fs-local"]
  }
}
```

#### 代码生成工具示例

#### 生成模型

```
帮我生成一个 User 模型，包含以下字段：
- name: string
- email: string
- age: integer
- is_active: boolean
```

Claude 会调用：
```json
{
  "tool": "loco_generate_model",
  "arguments": {
    "project_path": ".",
    "name": "user",
    "fields": {
      "name": "string",
      "email": "string",
      "age": "integer",
      "is_active": "boolean"
    },
    "with_timestamps": true
  }
}
```

#### 生成脚手架

```
为 BlogPost 生成一个完整的 API 脚手架，包含 title（string）和 content（text）字段
```

Claude 会调用：
```json
{
  "tool": "loco_generate_scaffold",
  "arguments": {
    "project_path": ".",
    "name": "blog_post",
    "fields": {
      "title": "string",
      "content": "text"
    },
    "kind": "api",
    "with_timestamps": true
  }
}
```

#### 生成控制器

```
为现有的 User 模型生成一个 HTML 控制器，只需要 index 和 show 动作
```

Claude 会调用：
```json
{
  "tool": "loco_generate_controller_view",
  "arguments": {
    "project_path": ".",
    "name": "users",
    "actions": ["index", "show"],
    "kind": "html"
  }
}
```

## 字段类型

Loco 支持以下字段类型：

- `string` - 字符串
- `text` - 长文本
- `integer` / `int` - 整数
- `big_integer` / `bigint` - 大整数
- `float` - 浮点数
- `decimal` - 精确小数
- `boolean` / `bool` - 布尔值
- `date` - 日期
- `time` - 时间
- `datetime` / `timestamp` - 日期时间
- `uuid` - UUID
- `json` / `jsonb` - JSON 数据

## 脚手架类型

- **`api`** - REST API（JSON 响应）
- **`html`** - 服务器渲染的 HTML 视图
- **`htmx`** - HTMX 驱动的交互式视图

## 安全与合规

### 审计日志

所有 CLI 操作都会记录到审计日志中，包含：
- 时间戳
- 工具名称
- 参数哈希值
- 操作员身份
- 执行时间
- 成功/失败状态

### 审批验证

每个 CLI 工具都需要特定的审批角色：
- `migrate_db`: ops_lead, security_officer
- `rotate_keys`: security_officer, cto
- `clean_temp`: ops_lead

### 超时控制

- 默认超时：60 秒
- 最大超时：300 秒
- 环境特定超时可通过配置覆盖

### 环境隔离

支持多环境配置：
- `development` - 开发环境
- `staging` - 测试环境
- `production` - 生产环境

## 开发

### 运行测试

```bash
pytest tests/
```

### 日志调试

设置环境变量启用详细日志：

```bash
export LOG_LEVEL=DEBUG
python -m loco_mcp_server.server
```

### 手动测试 MCP 协议

使用 MCP Inspector：

```bash
npx @modelcontextprotocol/inspector python -m loco_mcp_server.server
```

### 审计日志验证

检查审计日志：

```bash
tail -f /var/log/loco-mcp/audit.log
```

### 配置验证

验证环境配置：

```bash
echo $LOCO_MCP_AUDIT_LOG_PATH
echo $LOCO_MCP_DEFAULT_TIMEOUT
echo $LOCO_MCP_ENVIRONMENT
```

## 架构

```
┌─────────────────┐
│  Claude / AI    │
│    Assistant    │
└────────┬────────┘
         │ MCP Protocol
         │ (stdio)
┌────────▼────────┐
│  loco-mcp-      │
│     server      │ Python
│  (MCP Server)   │
└────────┬────────┘
         │ PyO3 Bindings
         │
┌────────▼────────┐
│  loco-bindings  │ Rust
│   (Python FFI)  │
└────────┬────────┘
         │
┌────────▼────────┐
│   loco-gen      │ Rust
│  (Generator)    │
└─────────────────┘
```

## 相关链接

- [Model Context Protocol](https://modelcontextprotocol.io)
- [MCP Python SDK](https://github.com/modelcontextprotocol/python-sdk)
- [Loco Framework](https://loco.rs)
- [loco-gen Documentation](../../docs)

## 许可证

MIT OR Apache-2.0
