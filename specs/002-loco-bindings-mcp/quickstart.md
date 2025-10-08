# Quickstart: MCP Tool Migration via Loco Bindings

本指南帮助运维工程师在 MCP 环境中验证已迁移的 CLI 工具，并确认与原终端体验一致。

## 前提条件
- 已部署最新版本的 `loco-mcp-server`，并通过 `.env` 配置自动化服务账号
- MCP 客户端（Cursor/Claude 等）已连接到 `loco-mcp` 服务器
- 目标 CLI 工具在 `cli-utilities.inventory.yaml` 中标记为 `priority <= 3`，且列出了审批角色
- 已配置 `/var/log/loco-mcp/audit.log` 审计日志路径

## 步骤 1：审阅工具清单
1. 打开 `specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml`
2. 确认目标 CLI 工具具备下列字段：`owner`、`usage_frequency`、`risk`、`approvals`、`timeout`、`guardrails`
3. 记录需要同时执行的审批操作和超时限制

## 步骤 2：验证 MCP 工具可发现性
1. 在 MCP 客户端中运行指令：`list_tools`
2. 确认以下工具出现在列表中：
   - `migrate_db` - 数据库迁移工具
   - `rotate_keys` - 密钥轮换工具  
   - `clean_temp` - 临时文件清理工具
3. 验证每个工具的描述、参数和超时设置与库存一致

## 步骤 3：执行测试调用

### 3.1 数据库迁移测试
```json
{
  "name": "migrate_db",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "development",
    "approvals": ["ops_lead", "security_officer"],
    "timeout_seconds": 60,
    "dependencies": ["postgres", "redis"]
  }
}
```

### 3.2 密钥轮换测试
```json
{
  "name": "rotate_keys",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "production",
    "approvals": ["security_officer", "cto"],
    "timeout_seconds": 300,
    "dependencies": ["kms"]
  }
}
```

### 3.3 临时文件清理测试
```json
{
  "name": "clean_temp",
  "arguments": {
    "project_path": "/path/to/loco/project",
    "environment": "development",
    "approvals": ["ops_lead"],
    "timeout_seconds": 60,
    "dependencies": ["fs-local"]
  }
}
```

## 步骤 4：CLI 等价性验证

### 4.1 迁移命令对比
```bash
# CLI 命令
cargo loco db migrate --environment development

# MCP 调用结果应产生相同的：
# - 数据库迁移状态
# - 错误消息格式
# - 执行时间范围
```

### 4.2 任务执行对比
```bash
# CLI 命令
cargo loco task rotate-keys --environment production

# MCP 调用结果应产生相同的：
# - 密钥轮换状态
# - 审批验证结果
# - 审计日志条目
```

## 步骤 5：审计与异常处理

### 5.1 审计日志验证
1. 检查 `/var/log/loco-mcp/audit.log` 中是否记录对应调用
2. 验证日志条目包含：
   - 时间戳
   - 工具名称
   - 参数哈希值
   - 操作员身份
   - 执行时间
   - 成功/失败状态

### 5.2 超时处理验证
1. 测试超时场景（设置极短超时时间）
2. 确认看到 `超时` 提示
3. 验证后台进程已终止
4. 检查审计日志记录超时事件

### 5.3 错误恢复流程
1. 如需重新执行，多步骤流程需按工具清单提供的顺序逐个调用
2. 验证中断后的状态恢复
3. 确认依赖检查正常工作

## 步骤 6：执行保证记录

### 6.1 校验和计算
```bash
# 计算 CLI 输出校验和
cargo loco db migrate --environment development > cli_output.txt
sha256sum cli_output.txt

# 计算 MCP 输出校验和  
# (从 MCP 响应中提取关键输出)
echo "mcp_output" > mcp_output.txt
sha256sum mcp_output.txt
```

### 6.2 记录验证结果
在 `Execution Assurance Record` 中新增条目：
- `cli_id`: "migrate-db"
- `verification_run_id`: "run_$(date +%Y%m%d_%H%M%S)"
- `expected_checksum`: CLI 输出校验和
- `actual_checksum`: MCP 输出校验和
- `status`: "pass" | "fail" | "waived"
- `tester`: 操作员姓名
- `run_timestamp`: 当前时间

## 步骤 7：配置验证

### 7.1 环境配置检查
```bash
# 验证环境变量配置
echo $LOCO_MCP_AUDIT_LOG_PATH
echo $LOCO_MCP_DEFAULT_TIMEOUT
echo $LOCO_MCP_ENVIRONMENT
```

### 7.2 权限验证
```bash
# 确认审计日志目录权限
ls -la /var/log/loco-mcp/
# 应显示：drwxr-xr-x 2 loco-mcp loco-mcp

# 确认服务账号权限
whoami
# 应显示配置的自动化服务账号
```

## 完成验证清单

- [ ] 所有三个 MCP 工具在 `list_tools` 中可见
- [ ] 每个工具的参数验证正常工作
- [ ] 超时机制按预期工作
- [ ] 审计日志正确记录所有调用
- [ ] CLI 和 MCP 输出校验和匹配
- [ ] 错误处理和恢复流程正常
- [ ] 审批序列验证正确
- [ ] 环境配置加载正常

完成以上步骤后，即可将该 CLI 工具标记为"通过 MCP 迁移验证"，并通知审批人签字。

