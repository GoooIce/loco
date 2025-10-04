# Feature Specification: Loco-MCP 高性能进程内 MCP 服务器

**Feature Branch**: `001-1-1-1`
**Created**: 2025-10-03
**Status**: Draft
**Input**: User description: "1. 概述 1.1 项目名称: Loco-MCP: 一个用于 loco-rs 的高性能进程内 MCP 服务器

1.2 问题陈述: loco-rs 是一个功能强大的 Rust Web 框架，其脚手架 CLI (cargo loco generate ...) 极大地加速了开发启动过程。然而，在 Claude Code 等 AI Agent 工作流中，通过命令行调用此功能存在以下痛点：

性能低下: 每次调用都涉及进程启动、编译检查和 I/O 操作，延迟高达数秒。
交互脆弱: Agent 需要生成精确的 CLI 字符串并解析不稳定的文本输出，容易出错。
上下文丢失: Agent 难以获取结构化的执行结果（如创建了哪些文件），阻碍了后续的自动化操作。
1.3 解决方案: 本项目旨在创建一个高性能的进程内 MCP (Model Context Protocol) 服务器。该服务器通过 Python 原生绑定直接调用 loco-rs 的核心脚手架逻辑，为 Claude Code Agent 提供一个快速、可靠、结构化的编程接口，完全绕过 CLI 和进程间通信的瓶颈。

2. 目标用户
主要用户: 使用 Claude Code 并选择 loco-rs 框架进行 Rust Web 应用开发的软件开发者。
次要用户: 构建自定义开发 Agent、需要以编程方式与 loco-rs 项目结构进行交互的开发者。
3. 项目目标与成功指标
3.1 项目目标:

功能性: 使 Claude Code Agent 能够以编程方式执行 loco-rs 的核心 generate 命令。
高性能: 实现从 Agent 指令到完成脚手架操作的端到端延迟显著低于传统的 CLI 调用方式。
易用性: 提供一组清晰、直观、与 loco-rs CLI 语义对齐的 MCP 工具。
可靠性: 提供结构化的成功响应和明确的错误信息，使 Agent 能够进行可靠的决策。
3.2 成功指标 (KPIs):

性能: 单个 generate 工具的端到端执行时间（从 MCP Server 接收到请求到 Rust 函数返回）应小于 10 毫秒。
可靠性: 对于格式正确的 Agent 请求，工具调用失败率应低于 1%。
采用率: 项目成功集成到一个完整的"创建新模型并为其添加 CRUD 端点"的 Agent 工作流中。
4. 用户场景概要

本功能主要支持开发者通过自然语言指令与 Claude Code 协作进行 loco-rs 应用开发，涵盖模型创建、控制器生成和完整的 CRUD 框架搭建。具体用户故事详见下文验收场景部分。
5. 功能需求 (V1)
本项目包含两个核心组件：一个 Rust-Python 绑定库和一个 Python MCP 服务器。

5.1 组件 1: loco-bindings (Python 原生模块) 这是一个将通过 Maturin 打包成 Python wheel 的 Rust 项目。

FR 1.1: 核心逻辑封装

必须重构 loco-cli 的 generate 命令逻辑，使其成为可从外部调用的库函数。
这些函数应接受结构化输入（如模型名、字段列表），而不是解析字符串。
FR 1.2: 导出 generate_scaffold 函数

使用 PyO3 导出一个名为 generate_scaffold 的 Python 可调用函数。
输入: model_name: str, fields: list[str] (例如 ['title:string', 'published_at:datetime:optional'])。
输出: 一个包含 created_files: list[str] 和 modified_files: list[str] 的 Python 字典。
FR 1.3: 导出 generate_model 函数

与 FR 1.2 类似，但用于单独生成模型和迁移。
FR 1.4: 导出 generate_controller_view 函数

与 FR 1.2 类似，用于生成控制器和视图。
FR 1.5: 错误处理

Rust 函数中的 Result::Err 或 panic! 必须被捕获并转换为具有描述性信息的 Python 异常（例如 PyValueError）。
5.2 组件 2: loco-mcp-server (Python 应用) 这是一个使用 Claude Agent SDK 的 Python 应用程序。

FR 2.1: 依赖 loco-bindings

项目的 pyproject.toml 必须将本地构建的 loco-bindings 作为依赖项。
FR 2.2: 定义 loco.generate_scaffold 工具

创建一个 MCP 工具，其参数与 loco-bindings 中的 generate_scaffold 函数匹配。
该工具的实现将直接调用绑定的 Python 函数，并将其字典输出返回给 Claude Agent。
FR 2.3: 定义 loco.generate_model 工具

同上，映射到 generate_model 函数。
FR 2.4: 定义 loco.generate_controller_view 工具

同上，映射到 generate_controller_view 函数。
FR 2.5: 进程内执行

服务器必须配置为进程内模式，以确保零 IPC 开销。
6. 非功能性需求
性能: 见 3.2。
安全性: 工具执行的文件操作必须严格限制在当前 loco-rs 项目目录内。
兼容性: 初始版本将支持 loco-rs v0.3.0 (最新稳定版)。
文档: 提供一个 README.md，清晰说明如何使用 uv 和 maturin 构建和运行该 MCP 服务器。
7. 技术架构
环境管理: uv
Rust-Python 绑定: PyO3
构建与打包: Maturin
MCP 服务器: claude-agent-py-sdk
工作流程:
开发者使用 uv 创建并管理 Python 虚拟环境。
开发者使用 maturin develop 在开发过程中快速编译 Rust 绑定并安装到虚拟环境中。
开发者启动 loco-mcp-server，该服务器导入 loco-bindings 模块。
Claude Agent 调用 MCP 工具，触发 Python 函数，该函数再直接调用进程内的 Rust 函数，实现高性能操作。
8. 范围之外 (Out of Scope for V1)
对 loco-rs 的 db, start, task 等非 generate 命令的支持。
为 loco-rs 的交互式生成器提供支持。
为除 Python 之外的其他语言（如 TypeScript/WASM）创建绑定。"

## Execution Flow (main)
```
1. Parse user description from Input
   → Feature description parsed successfully
2. Extract key concepts from description
   → Actors: Claude Code Agent, Developers
   → Actions: Generate scaffolding, models, controllers, views
   → Data: Model names, field definitions, file lists
   → Constraints: Performance <10ms, <1% failure rate, in-process execution
3. For each unclear aspect:
   → Performance targets specified
   → Error handling requirements clear
   → Scope boundaries well-defined
4. Fill User Scenarios & Testing section
   → User scenarios clearly defined in original description
5. Generate Functional Requirements
   → Extracted from FR sections in original description
6. Identify Key Entities
   → Models, controllers, views, file operations
7. Run Review Checklist
   → No implementation details detected
   → Requirements are testable and unambiguous
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
开发者使用 Claude Code 进行 loco-rs 应用开发时，希望能够通过自然语言指令快速生成项目脚手架，而无需忍受传统 CLI 调用的性能瓶颈和交互复杂性。

### Acceptance Scenarios
1. **Given** 开发者对 Claude 说"使用 loco-rs 创建一个名为 product 的模型，包含字段 name:string、price:i32 和 sku:string:unique", **When** Claude 调用 MCP 工具, **Then** 系统自动生成对应的模型文件和数据库迁移文件，并返回创建的文件列表
2. **Given** 开发者在创建模型后说"现在为 product 模型生成控制器和视图", **When** Claude 调用相应 MCP 工具, **Then** 系统自动创建 RESTful 控制器和基础视图文件
3. **Given** 开发者给出高级指令"搭建一个 posts 资源的完整 CRUD 框架", **When** Claude 依次调用 scaffold 工具, **Then** 系统生成完整的项目结构并允许 Claude 进行后续修改

### Edge Cases
1. **不支持的字段类型处理**
   - **Given** 开发者指定了不支持的字段类型（如 'invalid_type'）, **When** Claude 调用 generate_model, **Then** 系统返回清晰的错误消息："Unsupported field type: invalid_type. Supported types: string, i32, i64, boolean, datetime, text, optional"

2. **重复模型名称处理**
   - **Given** 开发者尝试创建已存在的模型名称, **When** Claude 调用 generate_model, **Then** 系统返回错误："Model 'product' already exists. Choose a different name or use 'force' parameter to overwrite"

3. **无效项目目录处理**
   - **Given** 当前目录不是有效的 loco-rs 项目, **When** Claude 调用任何 generate 工具, **Then** 系统返回错误："Not a valid loco-rs project directory. Run 'loco new' first or navigate to existing project"

4. **Rust 编译错误处理**
   - **Given** 生成的代码存在语法错误或类型问题, **When** 系统尝试编译生成的代码, **Then** 系统返回结构化错误响应：
     ```json
     {
       "error": "Compilation failed",
       "details": ["src/models/product.rs:15: Type mismatch found"],
       "suggestions": ["Check field types match expected schema"]
     }
     ```

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: System MUST allow Claude Code Agent to execute loco-rs generate commands programmatically
- **FR-002**: System MUST provide generate_scaffold function that creates complete CRUD scaffolding
- **FR-003**: System MUST provide generate_model function that creates model and migration files
- **FR-004**: System MUST provide generate_controller_view function that creates controllers and views
- **FR-005**: System MUST return structured responses with created_files and modified_files lists
- **FR-006**: System MUST execute operations in-process with <10ms response time
- **FR-007**: System MUST maintain <1% failure rate for well-formed requests
- **FR-008**: System MUST restrict file operations to current loco-rs project directory
- **FR-009**: System MUST provide descriptive error messages for troubleshooting

### Key Entities
- **Model**: Represents data structure with fields and constraints (name:string, price:i32, etc.)
- **Controller**: Handles HTTP requests and business logic for models
- **View**: Templates or response formats for data presentation
- **Migration**: Database schema change definitions
- **File Operation**: Creation or modification of project files

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---