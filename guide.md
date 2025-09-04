# Loco 中文使用指南（深入实践版）

> 面向 Rust 开发者的“Rust on Rails” 开发手册，覆盖从安装、脚手架、目录结构、配置、控制器/模型/作业/调度/存储/缓存/邮件，到测试与部署的端到端实践。基于当前仓库的源码结构与 CLI 行为整理，兼顾上手与生产落地。

- 适读人群：有 Rust 基础的后端/全栈工程师；Rails/Django/Spring 等背景迁移到 Rust 的团队；希望快速落地 Web/API/SaaS 服务的个人与企业。
- 配套资源：
  - 官方文档站（教程、专题与 API 参考）：[`https://loco.rs`](https://loco.rs)
  - 本仓库多语言 README 与示例模板（SaaS 起步、前端集成、代码生成器、部署脚手架）。

---

## 目录

1. Loco 是什么：理念与能力概览  
2. 安装与环境准备  
3. 快速开始：创建与启动你的第一个应用  
4. 项目目录结构与约定  
5. 配置系统与多环境管理  
6. 运行与开发体验（命令、路由、诊断）  
7. 控制器、路由与中间件  
8. 模型与数据库（迁移/实体/种子/多库）  
9. 身份认证与会话（SaaS 模板）  
10. 视图与资产（SSR 与 CSR 前端）  
11. 后台任务（Workers）与作业队列  
12. 任务调度（Scheduler）  
13. 邮件系统（Mailers）  
14. 存储（本地/云端/策略）  
15. 缓存（Cache）  
16. 日志、诊断与可观测性  
17. 代码生成器（Generators）与模板覆盖  
18. 前端集成（SaaS 前端）  
19. 测试与质量保障  
20. 部署与上线（Docker/Nginx/Shuttle）  
21. 迁移路径（面向 Axum/Rails 用户）  
22. 常见问题与故障排查（FAQ）  
23. 实战附录 A：从 0 到 1 的小型 SaaS（注册登录、文件上传、定时报表与邮件）  
24. 实战附录 B：多数据库与读写分离  
25. 实战附录 C：生产级部署参考与安全基线  

---

## 1. Loco 是什么：理念与能力概览

- Rust on Rails：延续“约定优于配置”，减少样板代码，以更少心智成本构建生产级应用。
- 架构选型：
  - Web 层：基于 Axum，高性能、可组合。
  - 数据层：整合 SeaORM，定义实体、关系与查询，无需大量 SQL；支持迁移与种子。
  - 视图层：可对接模板引擎（如 Tera）以进行服务端渲染（SSR）。
  - 资产层：支持客户端渲染（CSR），将前端构建产物由后端静态服务。
  - 后台能力：后台作业（多种队列与执行模式）、任务调度（更优雅的 cron 方案）。
  - 配套设施：邮件、文件存储（本地/云端）、缓存、日志、诊断与部署脚手架。
- 目标：
  - 快速开发：脚手架与生成器帮助你在几分钟内站起可运行的应用。
  - 强类型安全：Rust 生态下的稳健与性能。
  - 可扩展可替换：中间件与驱动松耦合，容易替换与扩展。

---

## 2. 安装与环境准备

- 安装 Loco（框架命令集经由 Cargo 子命令暴露）：

```sh
cargo install loco
```

- 可选：数据库工具（用于生成实体等）

```sh
cargo install sea-orm-cli
```

- 前提：
  - 已安装 Rust stable toolchain 与 Cargo。
  - 如启用 Postgres，请准备可连通的实例；如启用作业队列（Redis/Valkey/PG/Sqlite），亦需准备相应服务。
  - 如选择前端 `clientside`，需 Node.js 与包管理器（npm/pnpm）；本指南 SaaS 前端示例默认使用 pnpm。

---

## 3. 快速开始：创建与启动你的第一个应用

- 交互式向导：

```sh
loco new
```

将依次询问：
- 应用名（校验首字符与字符集合法性、不允许已存在目标目录）
- 模板：SaaS（服务端/客户端）、REST API、轻量服务、高级模式
- 数据库：`sqlite | postgres | none`
- 后台任务：`async | queue | blocking | none`
- 资产：`serverside | clientside | none`

- 非交互模式（一次性指定）：

```sh
loco new \
  --path . \
  --name myapp \
  --db sqlite \
  --bg async \
  --assets serverside \
  --os macos
```

- 进入项目并启动：

```sh
cd myapp
cargo loco start
```

默认开发环境监听 `http://localhost:5150`。如选择 `clientside`，需先进入 `frontend/` 安装依赖并构建产物后由后端服务静态资源。

---

## 4. 项目目录结构与约定

典型结构（不同模板会略有差别）：

- `src/`
  - `app.rs`、`boot.rs`、`config.rs`：应用装配、启动流程与配置入口
  - `controller/`：控制器与路由注册（基于 Axum）
  - `model/`：数据模型、查询 DSL、分页模块
  - `mailer/`：邮件模板与发送逻辑
  - `bgworker/`：后台作业执行器（Tokio/Redis/PG/Sqlite）
  - `scheduler.rs`：任务调度
  - `storage/`：文件存储驱动与策略（本地/云厂商/内存等）
  - `cache/`：缓存抽象与驱动
  - `validation.rs`：请求校验
- `config/`：多环境配置（`development.yaml`、`production.yaml` 等，均为 Tera 模板可插值）
- `migration/`：数据库迁移（启用 DB 时）
- `frontend/`：客户端渲染前端工程（启用 `clientside` 时）
- `assets/` 与模板目录：服务端渲染资源（启用 `serverside` 时）
- `tests/`：集成/单元测试与快照

“约定优于配置”意味着大部分功能只需在预期目录下扩展与实现即可被装配与发现。

---

## 5. 配置系统与多环境管理

- 每个配置文件均是合法 Tera 模板，支持变量替换与条件分支，灵活实现“同一套模板多环境落地”。
- 常见配置域：
  - `server`：`binding`、`port`、`middlewares`（静态资源、压缩、CORS、安全头、ETag、限载荷等）
  - `logger`：日志级别与输出格式
  - `database`：连接串、迁移策略（自动迁移或手动）
  - `queue`：队列提供者（Redis/PG/Sqlite/进程内 async）
  - `scheduler`：作业定义与触发规则
  - `storage`：默认驱动、本地路径、云端凭证与桶
  - `cache`：驱动、命名空间与过期策略
- 切换环境：

```sh
cargo loco start --environment production
```

也可以通过环境变量或容器参数注入环境选择与配置项值（安全项建议通过外部注入）。

---

## 6. 运行与开发体验（命令、路由、诊断）

- 启动与模式：

```sh
cargo loco start
cargo loco start --worker                # 仅 worker；可加标签: --worker=tag1,tag2
cargo loco start --server-and-worker     # 同进程服务 + worker
cargo loco start --all                   # 服务 + worker + scheduler
cargo loco start --binding 0.0.0.0 --port 8080 --no-banner
```

- 热重载（需安装 cargo-watch）：

```sh
cargo loco watch
```

- 路由与中间件可视：

```sh
cargo loco routes
cargo loco middleware --config
```

- 健康诊断与版本：

```sh
cargo loco doctor
cargo loco doctor --config
cargo loco version
```

- 自定义任务（见“任务与调度”章节）：

```sh
cargo loco task my_task foo:bar x:y
```

---

## 7. 控制器、路由与中间件

- 基于 Axum 的控制器与路由注册：更轻量的组合方式，提供请求提取器（路径参数、查询参数、JSON body 等）、校验、响应渲染。
- 组织方式：
  - `controller/app_routes.rs`：集中路由装配与分发
  - 按业务域拆分控制器文件，保持清晰边界
- 中间件（常见内置能力）：
  - 请求日志与追踪、请求 ID、远端 IP 解析
  - 压缩、ETag、CORS、安全响应头
  - 限制载荷大小、Panic 捕获、静态资源服务
- 使用 `cargo loco middleware --config` 查看启用状态与详细配置，按需在配置中开启/关闭或调整参数。
- 请求校验与错误响应：结合 `validation` 与统一错误结构，减少重复代码。

---

## 8. 模型与数据库（迁移/实体/种子/多库）

- 使用 SeaORM 定义实体与关系，避免直接手写大量 SQL。
- 迁移命令：

```sh
cargo loco db migrate        # 向上迁移
cargo loco db down 2         # 回滚 2 个版本
cargo loco db reset          # 清库并重跑迁移（慎用）
cargo loco db status
cargo loco db schema         # 导出 schema
cargo loco db truncate       # 清表数据（不删表）
```

- 种子与数据导出：

```sh
cargo loco db seed                 # 执行默认种子
cargo loco db seed --reset         # 清空数据后再种子
cargo loco db seed --dump          # 导出所有表
cargo loco db seed --dump-tables users,posts
```

- 实体生成（开发模式可用）：

```sh
cargo loco db entities
```

- 多数据库：适用于读写分离、多源/多租户等；通过 `initializers/multi_db.rs` 等扩展点装配连接与路由策略。

---

## 9. 身份认证与会话（SaaS 模板）

- SaaS 起步模板内置 `User` 模型与 JWT 认证：
  - JWT 可通过 Header 或 Cookie 传递（可配置位置与键名）
  - 登录/注册/刷新令牌等控制器动作可按需扩展
  - 中间件/提取器自动解析校验，将用户上下文注入到请求处理流程
- 生产建议：
  - JWT 私钥/密钥通过安全方式注入
  - Cookie 策略结合 Secure、HttpOnly、SameSite 与 HTTPS 环境
  - 对敏感接口增加速率限制与审计日志

---

## 10. 视图与资产（SSR 与 CSR 前端）

- 服务端渲染（SSR）：
  - 对接模板引擎（如 Tera），渲染动态 HTML
  - 适合后台管理与 SEO 友好场景
- 客户端渲染（CSR）：
  - `frontend/` 前端工程（SaaS 前端默认 TypeScript + React + Rsbuild + Biome）
  - 本地开发：

```sh
cd frontend
pnpm install
pnpm dev
```

  - 构建并由后端服务静态资源：

```sh
pnpm build
cargo loco start
```

- 资产中间件配置：指定静态目录与 fallback 策略（避免前端路由 404）。

---

## 11. 后台任务（Workers）与作业队列

- 模式：
  - `async`：进程内 Tokio 异步任务（简单快速）
  - `queue`：Redis/Valkey/PG/Sqlite 队列驱动（跨进程扩展）
  - `blocking`：同步阻塞（演示/一次性任务）
- 运行与管理：

```sh
cargo loco start --worker                 # 仅 worker，或传标签：--worker=email,report
cargo loco jobs cancel --name job_name
cargo loco jobs tidy                      # 清理已完成/已取消
cargo loco jobs purge --max-age 90 --dump # 清理超龄并可导出快照
cargo loco jobs dump --status completed --folder ./jobs
cargo loco jobs import --file ./jobs/xxx.json
cargo loco jobs requeue --from-age 15
```

- 生产建议：
  - 使用 `queue` 模式并独立部署若干 worker 实例
  - 用标签区分不同类型作业（邮件、报表、视频处理等），并分配到不同 worker 组
  - 设计重试/幂等与死信队列策略，避免雪崩与重复处理

---

## 12. 任务调度（Scheduler）

- 用统一配置替代分散的 crontab，便于版本化与观测：

```sh
cargo loco scheduler             # 按环境默认配置执行
cargo loco scheduler --list      # 查看作业列表
cargo loco scheduler --name daily_report
cargo loco scheduler --tag analytics
cargo loco scheduler --config config/scheduler.yaml
```

- 建议与应用进程解耦（或使用 `--all` 在同进程联合启动），将调度与 worker 组合实现稳定的周期作业执行。

---

## 13. 邮件系统（Mailers）

- 通过 Mailer 定义发送逻辑与模板，实际投递由后台作业负责，避免阻塞请求链路。
- 配置 SMTP/服务商访问凭证，区分开发与生产环境行为（开发可走本地预览或假投递）。
- 测试：对模板与发送逻辑进行快照测试，保障回归稳定。

---

## 14. 存储（本地/云端/策略）

- 驱动：内存（inmem）、本地磁盘（local）、云端（AWS S3、GCP、Azure 等）与空实现（null）。
- 策略：
  - `single`：单目标存储
  - `mirror`：多目标镜像写入，提升可用性
  - `backup`：主存储 + 备份容灾
- 场景实践：用户头像/文档上传、日志/报表导出、媒体处理；结合生命周期/权限策略保障安全与成本。

---

## 15. 缓存（Cache）

- 驱动：内存、Redis 等。
- 用法：热点数据与接口响应加速、幂等控制、限流辅助。
- 建议使用统一命名空间与序列化规范，避免键冲突与升级兼容问题。

---

## 16. 日志、诊断与可观测性

- 日志级别：`ERROR | WARN | INFO | DEBUG | TRACE`，可按环境与 CLI 参数调整。
- 路由与中间件清单：便于核对装配与排查请求路径。
- 诊断：

```sh
cargo loco doctor
cargo loco doctor --config
```

- 建议在生产启用结构化日志与集中收集（如 JSON），并引入指标/追踪方案。

---

## 17. 代码生成器（Generators）与模板覆盖

- 仅在开发模式（debug assertions）可用：

```sh
cargo loco generate model posts title:string! user:references --api
cargo loco generate migration AddUserRefToPosts user:references
cargo loco generate scaffold posts --kind api
cargo loco generate controller posts --api list remove update
cargo loco generate task report_daily
cargo loco generate worker image_processor
cargo loco generate mailer welcome
cargo loco generate data init_catalog
cargo loco generate scheduler
cargo loco generate deployment --kind docker|nginx|shuttle
cargo loco generate override scaffold/api/controller.t
```

- 模板覆盖（override）：将内置模板复制到本地进行自定义；若不需要，删除本地覆盖目录即可回退。

---

## 18. 前端集成（SaaS 前端）

- 技术栈：TypeScript + React + Rsbuild + Biome。
- 开发：

```sh
cd frontend
pnpm install
pnpm dev
```

- 构建与后端静态服务：

```sh
pnpm build
cargo loco start
```

- 如不使用 React，可替换为其他前端库，Rsbuild 便于切换。

---

## 19. 测试与质量保障

- 覆盖层次：
  - 单元测试：模型/服务/工具
  - 集成测试：控制器路由、认证流程、作业/调度、存储驱动
  - 端到端（E2E）：联动前端与真实依赖
- 运行：

```sh
cargo test
```

- 数据库测试建议：
  - 使用测试库或内存驱动，隔离测试数据
  - 复用测试配置与工具（如 `tests_cfg`）构建稳定上下文
- 快照测试：对控制器响应/视图/邮件模板进行快照，便于回归对比。

---

## 20. 部署与上线（Docker/Nginx/Shuttle）

- Release 构建：

```sh
cargo build --release
# 二进制位于 target/release/
```

- 生成部署脚手架（开发模式）：

```sh
cargo loco generate deployment --kind docker
cargo loco generate deployment --kind nginx
cargo loco generate deployment --kind shuttle
```

- 典型拓扑：
  - Nginx 反向代理 + Loco 二进制
  - Docker 镜像 + 容器编排（K8s/Compose/Nomad）
  - Shuttle 云托管

- 生产配置基线：
  - 明确 `production.yaml` 的端口/绑定/日志级别
  - 数据库/队列/存储凭证通过环境安全注入
  - 前端静态资源构建并妥善挂载
  - 健康检查与滚动升级策略（灰度、回滚）

---

## 21. 迁移路径（面向 Axum/Rails 用户）

- Axum 开发者：可直接复用已有 Handler 与中间件思路，将控制器与路由装配到 Loco 的 `app_routes` 体系；配置系统与后台能力可作为“增强层”。
- Rails 开发者：约定优于配置、控制器/模型/视图的心智模型相似；不同点在于 Rust 的类型系统与异步生态、数据库与存储/队列驱动差异，需要适应 Rust 工具链与异步编程范式。

---

## 22. 常见问题与故障排查（FAQ）

- 在 Git 仓库中生成被阻止？
  - 交互确认或添加 `--allow-in-git-repo`。
- 前端构建失败？
  - 检查 Node/pnpm/npm；在 `frontend/` 内执行安装与构建；确认静态目录配置正确。
- 连接 Postgres/Redis 失败？
  - 检查连接串、网络与凭证；优先使用命令行客户端做连通性排查；区分开发与生产参数。
- 目录已存在导致生成器退出？
  - 更换 `--name` 或 `--path`，避免覆盖。
- macOS 用户有什么特别建议？
  - 显式传 `--os macos`，避免在 Unix 系统默认选择 `linux` 带来的差异。
- Worker 不消费任务或调度未执行？
  - 核对后台模式配置、标签筛选、队列连接与权限；使用 `jobs list/dump/purge/requeue` 等命令排查。
- 静态资源 404？
  - 检查静态中间件目录与 `fallback`，确保前端路由回退到入口页面。

---

## 23. 实战附录 A：从 0 到 1 的小型 SaaS（注册登录、文件上传、定时报表与邮件）

本附录以“用户注册/登录 + 文件上传 + 每日报表邮件”为例，串联 Loco 的核心能力。

### 23.1 初始化项目

- 模板选择：SaaS（服务端渲染或客户端渲染均可）
- 选项：`--db sqlite`、`--bg queue`（或 `async`）、`--assets serverside|clientside`

### 23.2 用户认证

- 使用内置 User 模型与 JWT，调整配置以选择 Token 位置（Header/Cookie）。
- 为需要鉴权的控制器装配提取器，中间件自动为请求注入用户上下文。

### 23.3 文件上传（Storage）

- 选择驱动：本地开发用 `local`，生产用 `aws|gcp|azure` 并结合 `mirror/backup` 提升可靠性。
- 控制器接收 multipart/form-data，将文件通过 `Storage` 抽象写入；返回存储键与访问地址。

### 23.4 后台作业：生成日报

- 定义作业（Worker）读取当天业务数据（例如新增用户/活跃数），生成 CSV 或 HTML 报告，写入 `Storage` 并发送邮件通知。
- 在 `scheduler` 中定义每日 9:00 任务，或手动触发 `cargo loco scheduler --name daily_report` 调试。

### 23.5 邮件（Mailer）

- 定义欢迎邮件与日报通知邮件模板；本地调试走假投递或本地预览；生产配置 SMTP/服务商。

### 23.6 端到端验证

- 单元测试模型与服务、快照测试控制器响应/模板，构造集成测试模拟用户注册→上传→生成报表→收件箱校验（可通过 dump 方式确认邮件内容）。

### 23.7 交付与部署

- 生成 Docker 与 Nginx 脚手架；完善 `production.yaml`；将凭证注入容器环境；上线验证路由、中间件、静态资源、队列与调度可用性。

---

## 24. 实战附录 B：多数据库与读写分离

- 需求：热点读多写少的大型应用；或多租户按库隔离。
- 做法：
  - 在初始化阶段装配多个数据库连接（主写库、只读副本或多租户库集合）。
  - 在数据访问层封装 Router，根据请求上下文或读写语义路由到不同连接；
  - 事务与一致性：明确只读事务与写事务边界，避免跨库事务；必要时用事件驱动或补偿机制。
- 测试：
  - 使用 `tests_cfg` 构造多库环境，覆盖路由策略与降级容错路径。

---

## 25. 实战附录 C：生产级部署参考与安全基线

- 镜像与二进制：
  - 多阶段构建减小镜像体积；使用非 root 运行用户；开启只读文件系统；挂载必要可写卷。
- 配置与凭证：
  - 一律通过环境安全注入；CI/CD 使用密钥管理服务；避免将凭证写入镜像或仓库。
- 网络与暴露面：
  - 仅暴露必要端口；前置反代（Nginx/ALB）启用 TLS 与 HTTP 安全头；
  - CORS 精确白名单；
  - 限流/速率限制与 DDoS 防护。
- 数据安全：
  - 数据库用户最小权限；启用加密传输；定期备份与演练；
  - 对象存储最小权限与临时凭证；
  - 日志脱敏（用户隐私、凭证、令牌）。
- 可观测性：
  - 结构化日志 + 日志聚合；指标（延迟、错误率、QPS、队列堆积）与告警；
  - 健康检查与金丝雀发布；回滚应急预案。

---

## 结语

通过本指南，你可以从 0 搭建并上线一个基于 Loco 的 Web/API/SaaS 应用。建议配合官方文档站的专题文章与示例仓库继续深入，在团队内建立一套“约定与骨架”，让新项目与新同事都能迅速对齐并高效产出。

更多资料与社区支持请访问：[`https://loco.rs`](https://loco.rs)
