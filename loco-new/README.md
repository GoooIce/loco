## Loco CLI 中文新手教程

Loco 是一套用 Rust 构建现代 Web/API 应用的框架与脚手架工具，提供数据库、后台任务、资源（前后端渲染）等常用能力的一站式初始化与配置。本文面向初学者，手把手带你从安装、创建项目到理解命令参数与向导流程。

更多参考可见 [Loco 官网](https://loco.rs) 与其文档。

### 安装

```sh
cargo install loco
```

安装后可在任意目录使用 `loco`。如果你之前已安装，重复执行上面命令即可升级到最新版。

### 快速开始（交互式向导）

```sh
loco new
```

执行后会进入交互式向导：

- **App 名称**：默认 `myapp`，需满足命名规则：
  - 不能为空；
  - 首字符不能为数字；
  - 首字符需为 Unicode XID 起始字符或下划线；
  - 后续字符需为 Unicode XID 连续字符、数字或 `-`。
- **要构建什么？模板选择**（五选一）：
  - SaaS（服务端渲染）：会启用服务端模板；
  - SaaS（客户端渲染）：会启用前端构建输出的客户端资产；
  - REST API（含数据库与用户认证）；
  - 轻量服务（仅控制器与视图，最小化）；
  - 高级模式（逐项自定义）。

随后根据模板不同，向导会引导你选择：

- **数据库（--db）**：`sqlite | postgres | none`
  - sqlite：开箱即用；
  - postgres：需具备可连接的 Postgres 实例；
  - none：不启用数据库。
- **后台任务（--bg）**：`async | queue | blocking | none`
  - async：进程内 Tokio 异步任务；
  - queue：基于 Redis/Valkey 的队列式独立 worker；
  - blocking：前台阻塞式执行（会阻塞请求，适合演示/简单任务）；
  - none：不启用后台任务（在“高级模式”下，如果你选择了数据库，后台任务不允许为 none）。
- **资源/资产（--assets）**：`serverside | clientside | none`
  - serverside：服务端渲染（会开启视图引擎初始化）；
  - clientside：前端资源由独立前端工程构建产出并由后端服务；
  - none：不配置资产。

生成完成后会提示成功并输出关键信息。如果选择了 `clientside`，终端会提醒：

```text
assets: You've selected `clientside` ...
Next step, build your frontend:
  $ cd frontend/
  $ npm install && npm run build
```

### 命令行用法（非交互）

除交互式向导外，所有选项均可通过参数直接指定：

```sh
loco new \
  --path . \
  --name myapp \
  --db sqlite \
  --bg async \
  --assets serverside \
  --os macos
```

参数说明：

- `-p, --path <PATH>`：生成到的父目录（默认 `.`）。最终会在该目录下创建子目录 `<app_name>`。
- `-n, --name <NAME>`：应用名；不传则进入交互式输入（有命名校验）。
- `--db <sqlite|postgres|none>`：数据库选择。
- `--bg <async|queue|blocking|none>`：后台任务选择。
- `--assets <serverside|clientside|none>`：资产配置。
- `-a, --allow-in-git-repo`：允许在当前 Git 仓库中生成（默认不允许，会提示确认）。
- `--os <linux|windows|macos>`：为目标系统优化生成（默认：在 Unix 上为 `linux`，在非 Unix 上为 `windows`；macOS 用户建议显式传 `--os macos`）。
- `-l, --log <ERROR|WARN|INFO|DEBUG|TRACE>`：日志/输出详细程度（默认 `ERROR`）。

注意：如果目标路径中 `<app_name>` 目录已存在，会直接报错退出。

### 生成后的目录与下一步

1) 进入目录并阅读项目自带 README：

```sh
cd <app_name>
```

2) 如果选择了 `clientside` 资产：

```sh
cd frontend
npm install
npm run build
```

3) 如果选择了 `postgres` 或 `queue`：

- postgres：确保本地或远端有可连接的 Postgres 实例；
- queue：确保本地或远端有可连接的 Redis/Valkey 实例。

4) 启动应用（在生成的项目内）：

```sh
cargo loco start
```

应用启动后会在本地端口监听（详见项目 `config/development.yaml`）。

### 向导逻辑一览（基于源码行为）

- 模板到选项的对应关系：
  - 轻量服务：`db=none`，`bg=none`，`assets=none`。
  - REST API：需选择 `db` 与 `bg`，`assets=none`。
  - SaaS（服务端渲染）：需选择 `db` 与 `bg`，`assets=serverside`。
  - SaaS（客户端渲染）：需选择 `db` 与 `bg`，`assets=clientside`。
  - 高级模式：`db`、`bg`、`assets` 均逐项选择；当 `db!=none` 时，`bg` 不可为 `none`。

- 其它行为细节：
  - 如果未传 `--name`，向导以“App name?” 提示并做命名校验；
  - 默认会尝试检测目标目录是否处在 Git 仓库内，若是且未加 `--allow-in-git-repo`，会弹出确认提示；
  - 生成完成后会尝试在生成目录内执行 `cargo fmt`（失败不会中断，仅提示）；
  - 生成路径为 `<path>/<app_name>`；若该目录已存在则直接报错退出；
  - 选择 `serverside` 时会在配置中启用视图引擎初始化；
  - 当 `db` 与 `bg` 均启用时，会额外启用认证/邮件等组件；
  - 环境变量 `LOCO_DEV_MODE_PATH` 仅用于本地开发 Loco 框架本体时的特殊指向，普通用户无需设置。

### 常见问题（FAQ）

- 生成器提示“在 Git 仓库中”，如何继续？
  - 交互确认即可，或加 `--allow-in-git-repo` 跳过提示。
- `frontend` 构建报错？
  - 确保本机 Node/npm 可用；进入 `frontend/` 执行 `npm install && npm run build`。
- 选择 `postgres`/`queue` 后启动失败？
  - 确认数据库/Redis 可联通；必要时修改项目内配置文件的连接串。
- 目录已存在导致失败？
  - 需更换应用名或目标路径，生成器不会覆盖已存在目录。
- macOS 用户需要注意什么？
  - 建议显式传 `--os macos`，避免默认在 Unix 上选择 `linux`。

——

当你完成以上步骤，就可以在本地打开浏览器访问项目并开始基于 Loco 开发了。遇到问题，可查阅官网文档或在社区中提问。