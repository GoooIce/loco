# Phase 0 Research: Expand MCP Tools via Loco Bindings

## Inventory Canonical Source
- **Decision**: Maintain the CLI inventory in a versioned manifest at `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml` that is generated from the existing `loco-cli` metadata and enriched with ownership details.
- **Rationale**: Keeping the inventory co-located with project specifications ensures reviewability, enables PR-based approvals, and keeps the MCP catalog synchronized with the authoritative CLI definitions.
- **Alternatives Considered**:
  - Store the list inside the Python MCP server package → rejected because it would duplicate source-of-truth data and violate DRY.
  - Query the CLI binary at runtime to discover commands → rejected because it introduces runtime latency and hides changes from review workflows.

## Prioritization Heuristics
- **Decision**: Rank CLI utilities by daily usage frequency, then break ties by operational risk (severity of failure) and compliance requirements, recording the rationale in the manifest.
- **Rationale**: Matches stakeholder guidance and ensures the highest-impact tools move first without ignoring governance constraints.
- **Alternatives Considered**:
  - First-in-first-out ticket queue → rejected because it ignores business impact.
  - Solely risk-based prioritization → rejected because some high-frequency but low-risk utilities still deliver outsized productivity gains.

## Multi-step Workflow Handling
- **Decision**: Model multi-step CLI workflows as orchestrated sequences of independent MCP tools, each representing a single CLI step with explicit prompts and outputs.
- **Rationale**: Aligns with clarification Session 2025-10-08 and keeps each MCP call atomic, auditable, and interruptible.
- **Alternatives Considered**:
  - Keep multi-stage prompts inside one MCP tool call → rejected because it complicates user guidance and makes error recovery ambiguous.
  - Build a bespoke state machine inside the bindings → rejected as it duplicates orchestration logic better handled by the client assistant.

## Long-running Command Policy
- **Decision**: Provide a configurable timeout per MCP tool (default 60 seconds, maximum 300) with deterministic termination messaging surfaced back to the assistant.
- **Rationale**: Meets the new FR-009 requirement while preventing runaway processes from blocking the MCP session.
- **Alternatives Considered**:
  - Unlimited runtime → rejected per operations SLOs and assistant UX guidelines.
  - Hard-coded single timeout value → rejected because different utilities have distinct performance envelopes.

## Authentication & Auditing
- **Decision**: Reuse the existing automation service accounts and inject credentials via environment configuration at server start; log every MCP invocation with tool name, parameters hash, and operator identity to `/var/log/loco-mcp/audit.log`.
- **Rationale**: Satisfies FR-007 and governance requirements without prompting users for sensitive data.
- **Alternatives Considered**:
  - Prompt users for credentials → rejected for security reasons.
  - Store credentials in manifest → rejected to avoid secrets in source control.

## Validation & Parity Testing
- **Decision**: For each MCP tool, record an `expected_output` checksum in the Execution Assurance Record and add pytest-based contract tests that assert the Python bindings call loco-gen with the exact parameters used by the CLI regression suite.
- **Rationale**: Ensures FR-005 parity and provides fast regression feedback when loco-gen evolves.
- **Alternatives Considered**:
  - Manual spot checks only → rejected due to high risk of drift.
  - Re-implement CLI logic in Python tests → rejected for violating DRY and increasing maintenance overhead.

## CLI Command Surface Mapping
- **Decision**: 将 `Commands::Db::Migrate` 作为首批 MCP 工具之一，对应清单中的 `migrate-db`，通过复用 `run_db::<H, M>` 路径触发 `RunDbCommand::Migrate`。
- **Rationale**: `src/cli.rs` 中的 `Db` 子命令已经封装了迁移流程（含 `db::create`、`run_db` 调用），直接绑定可确保与现有 CLI 模式完全一致，并复用 SeaORM 迁移校验。
- **Alternatives Considered**:
  - 在 MCP 端重写迁移逻辑 → 违反宪法的直接集成原则，且高风险。
  - 通过 shell 调用 `cargo loco db migrate` → 增加额外进程开销并弱化审计。

- **Decision**: 利用通用 `Commands::Task` + `task::Vars::from_cli_args` 路径暴露自定义维护任务（如 `rotate-keys`、`clean-temp`）。
- **Rationale**: `cli.rs` 中的 `Task` 分支会复用 `run_task::<H>`，允许通过名称调度任意注册任务。MCP 只需封装任务名称与键值参数，即可覆盖多个高频维护任务。
- **Alternatives Considered**:
  - 为每个任务单独实现 CLI 入口 → 会造成任务注册与 CLI 分离，维护成本过高。
  - 在 Python 层模拟任务执行 → 会绕过 Rust 侧校验与日志。

- **Decision**: 保留 `Cli`/`Playground` 的全局 `--environment` 选项，并在 MCP 参数中显式呈现该字段。
- **Rationale**: `Cli` 结构在顶层定义了 `environment` 字段，且 `main` 会基于该值加载配置。MCP 工具暴露同名可选输入即可与现有环境切换策略对齐。
- **Alternatives Considered**:
  - 固定使用默认环境 → 无法满足多环境合规测试需求。
  - 在 MCP 层重新解析 `.env` → 与 CLI 行为不一致并增加遗漏风险。

- **Decision**: 对多阶段作业使用 `Commands::Scheduler` 与 `Commands::Jobs` 组合，将长流程拆分为单步 MCP 工具。
- **Rationale**: `scheduler` 与 `jobs` 子命令在 CLI 内部已拆分不同操作 (`run_scheduler`, `handle_job_command`)。遵循 clarifications，将其映射为多工具序列可保持可审计性和可中断性。
- **Alternatives Considered**:
  - 将调度器功能封装为单个长流程 MCP 调用 → 难以提供进度反馈和失败重试。
  - 在 MCP 端实现自定义 orchestrator → 与项目“薄绑定”原则冲突。

- **Decision**: MCP 超时默认继承 CLI 同步行为（阻塞直至完成），并结合 `research.md` 既定策略增加 60s 默认、300s 上限的超时控制。
- **Rationale**: CLI 当前通过同步 `await` 调用执行命令，无显式超时。MCP 层添加超时可以满足 FR-009，同时保持 CLI 行为（超时即终止）。
- **Alternatives Considered**:
  - 修改 CLI 增加内部超时 → 需要改动核心 Rust 逻辑，风险高。
  - 无超时限制 → 违反新的操作要求。

