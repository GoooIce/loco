
# Implementation Plan: Expand MCP Tools via Loco Bindings

**Branch**: `002-loco-bindings-mcp` | **Date**: 2025-10-08 | **Spec**: `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/spec.md`
**Input**: Feature specification from `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/spec.md`

## Summary
本次增强要求将 `src/cli.rs` 中已有的高频命令行工具映射为 MCP 工具，使运维工程师能够在 MCP 工作区获得与 CLI 等价的体验并满足 FR-001~FR-009 约束。方案沿用现有 `loco-cli` 能力：优先覆盖数据库迁移（`Commands::Db::Migrate`）、高风险任务（通过 `Commands::Task` 触发注册任务）与多阶段作业（`Commands::Scheduler`/`Commands::Jobs` 拆解）。所有 MCP 工具仅做参数转换、环境选择、超时封装，复用 Rust 侧逻辑、审计和校验链路。

## Technical Context
**Language/Version**: Rust 1.75 (核心 CLI)、Python 3.11 (loco-mcp-server)  
**Primary Dependencies**: `clap`, `loco_gen`, `sea-orm-migration`, `async-std` / `tokio`（由项目现有依赖提供）、Python `mcp` SDK  
**Storage**: PostgreSQL / Redis（由 CLI 命令接入，MCP 端不直接存储）  
**Testing**: `cargo test`（Rust CLI 回归）、`pytest`（MCP 合约与契约测试）  
**Target Platform**: Linux/macOS 服务器（与现有 CLI 一致）  
**Project Type**: single（Rust 主仓库 + Python 辅助组件，不引入多项目拆分）  
**Performance Goals**: MCP 工具响应 < 60s 默认、< 300s 上限；生成类命令匹配 CLI 性能；无额外进程开销  
**Constraints**: 绑定层仅类型转换；强制超时终止与审计日志；复用自动化账号；计划需通过阅读 `/Users/devel0per/Code/framework/loco/src/cli.rs` 制定具体实现内容。  
**Scale/Scope**: 首批 3 个高优先级 CLI 工具及其多步骤流程（`migrate-db`、`rotate-keys`、`clean-temp`）

## Constitution Check
- **G1 Simplicity & DRY**: ✅ 计划直接复用 `Commands::Db`、`Commands::Task`、`Commands::Scheduler/Jobs` 中现有逻辑，不复制生成或校验代码。
- **G2 Thin Binding Layers**: ✅ MCP 层仅包装参数、timeout、输出格式；所有业务逻辑保留在 Rust CLI 与已注册任务中。
- **G3 Direct Integration**: ✅ 通过现有函数 `run_db::<H,M>`、`run_task::<H>`、`run_scheduler::<H>`、`handle_job_command::<H>` 调用 loco-gen 能力，无自建管线。
- **G4 Maintainability First**: ✅ 使用单一 specs 目录管理文档；MCP 新增工具继承现有任务注册机制，避免新模块或依赖。
- **G5 Performance Through Native Code**: ✅ 不引入缓存/线程池；MCP 请求同步调用 Rust 路径，仅增加可控超时。
- **G6 Workflow Discipline**: ✅ 计划在 Phase 1 定义契约及 pytest 合约测试；更新 quickstart 与审计策略；Phase 2 后将编写任务驱动测试优先顺序。

## Project Structure
### Documentation (this feature)
```
specs/002-loco-bindings-mcp/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-utilities.inventory.yaml
└── tasks.md  (由 /tasks 生成)
```

### Source Code (repository root)
```
src/
├── cli.rs                  # CLI 命令定义，MCP 工具绑定的唯一来源
├── task.rs                 # 任务参数解析，供 rotate-keys / clean-temp 复用
├── scheduler.rs            # 调度命令封装，支撑多步骤工具拆解
├── bgworker/               # 作业队列实现，对应 Jobs 子命令
└── app.rs                  # Hooks::register_tasks 注册 CLI 可用任务

loco-mcp/
├── loco-mcp-server/src/    # Python MCP 绑定层（调用 Rust FFI）
└── loco-mcp-server/tests/  # pytest 合约与集成测试

specs/002-loco-bindings-mcp/
└── contracts/              # CLI 清单 & MCP 契约输出位置
```

**Structure Decision**: 单项目结构；Rust 主仓库提供 CLI 能力，Python MCP 服务器作为绑定层，无需拆分前后端或移动端目录。

## Phase 0: Outline & Research
研究已完成（详见 `research.md`）：
- 梳理 CLI 命令面及绑定策略，确认 `Db::Migrate`、`Task`、`Scheduler/Jobs` 的映射方式。
- 确立优先级与多步骤拆解、超时控制、审计策略等关键决策。
- 所有 NEEDS CLARIFICATION 均已解决。

## Phase 1: Design & Contracts
1. **data-model.md**：巩固 `CLI Utility Profile`、`MCP Tool Listing`、`Execution Assurance Record` 结构，支持多 MCP 工具映射同一 CLI。
2. **contracts/**：
   - 维护 `cli-utilities.inventory.yaml`（含优先级、审批、依赖、风险）。
   - 记录每个 MCP 工具的参数、输出与默认超时说明，保持与 CLI 文档同步（无需生成 OpenAPI/JSON Schema，面向 stdio 协议）。
3. **合约测试**：
   - `loco-mcp-server/tests/test_tools.py` 新增用例，确保 MCP 调用触发 `loco_bindings` 对应函数，并验证参数映射与 timeout。
   - `test_scenarios.py` 扩展多步骤流程（e.g., scheduler + jobs）。
4. **quickstart.md**：
   - 编写操作手册，指导运维在 MCP 客户端执行三个工具、对比 CLI 输出、检查审计日志与校验记录。
5. **Agent 文件**：运行 `.specify/scripts/bash/update-agent-context.sh cursor`，记录新增技术点（Rust CLI 绑定策略、Python MCP timeout 配置等）。

## Phase 2: Task Planning Approach
- 使用 `/tasks` 命令基于 Phase 1 产出生成任务清单。
- 任务覆盖：更新 Python 绑定与测试、扩展 Rust 任务注册、完善 inventory 与契约、编写 quickstart 校验步骤。
- 顺序：先生成/更新测试（合约→集成），再调整实现。标记可并行项 `[P]`（例如不同测试文件）。

## Phase 3+: Future Implementation
- Phase 3：执行 tasks.md；
- Phase 4：实现并让测试通过；
- Phase 5：验证（运行 quickstart、检查审计/timeout 行为）。

## Complexity Tracking
| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| *None* | | |

## Progress Tracking
**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---
*Based on Constitution v1.1.0 - See `/Users/devel0per/Code/framework/loco/.specify/memory/constitution.md`*
