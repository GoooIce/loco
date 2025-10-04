# Loco MCP Server 使用示例

## 快速开始

### 1. 安装和启动

```bash
# 安装服务器
cd loco-mcp/loco-mcp-server
pip install -e .

# 在终端中直接运行（用于测试）
python -m loco_mcp_server.server
```

### 2. 在 Claude Desktop 中使用

配置 Claude Desktop 的 `claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "loco": {
      "command": "python",
      "args": ["-m", "loco_mcp_server.server"]
    }
  }
}
```

重启 Claude Desktop 后，你会在工具栏看到 🔌 图标，表示 MCP 服务器已连接。

### 3. 使用工具

## 示例 1: 创建用户模型

**对 Claude 说：**
```
帮我在当前目录的 Loco 项目中创建一个 User 模型，包含：
- username: string
- email: string  
- password_hash: string
- is_admin: boolean
```

**Claude 会执行：**
```json
{
  "tool": "loco_generate_model",
  "arguments": {
    "project_path": ".",
    "name": "user",
    "fields": {
      "username": "string",
      "email": "string",
      "password_hash": "string",
      "is_admin": "boolean"
    },
    "with_timestamps": true
  }
}
```

**生成的文件：**
- `src/models/user.rs` - 模型定义
- `migration/src/mYYYYMMDDHHMMSS_user.rs` - 数据库迁移

## 示例 2: 生成博客文章脚手架

**对 Claude 说：**
```
为我的博客系统生成 BlogPost 脚手架，使用 HTML 模式，包含：
- title: string
- content: text
- published: boolean
- author_id: integer
```

**Claude 会执行：**
```json
{
  "tool": "loco_generate_scaffold",
  "arguments": {
    "project_path": ".",
    "name": "blog_post",
    "fields": {
      "title": "string",
      "content": "text",
      "published": "boolean",
      "author_id": "integer"
    },
    "kind": "html",
    "with_timestamps": true
  }
}
```

**生成的文件：**
- `src/models/blog_post.rs` - 模型
- `src/controllers/blog_posts.rs` - 控制器（CRUD 操作）
- `migration/src/mYYYYMMDDHHMMSS_blog_post.rs` - 迁移
- `assets/views/blog_posts/*.html` - 视图模板

## 示例 3: 为现有模型添加控制器

**对 Claude 说：**
```
我已经有一个 Product 模型了，帮我生成 API 控制器，只需要这些动作：
- list 列表
- show 详情
- create 创建
```

**Claude 会执行：**
```json
{
  "tool": "loco_generate_controller_view",
  "arguments": {
    "project_path": ".",
    "name": "products",
    "actions": ["index", "show", "create"],
    "kind": "api"
  }
}
```

**生成的文件：**
- `src/controllers/products.rs` - API 控制器（JSON 响应）

## 进阶用法

### 复杂字段类型

```
创建 Order 模型：
- order_number: uuid
- total_amount: decimal
- order_data: jsonb
- created_at: timestamp
- shipped_at: datetime
```

### 关系模型

```
创建 Comment 模型，关联到 BlogPost：
- blog_post_id: integer (外键)
- user_id: integer (外键)
- content: text
- is_approved: boolean
```

### HTMX 脚手架

```
生成一个支持 HTMX 的 Task 脚手架：
- title: string
- description: text
- status: string
- priority: integer

使用 htmx 模式，实现局部刷新和动态交互
```

## 工作流程示例

### 构建完整的博客系统

1. **创建基础模型**
```
创建 User 模型：username, email, password_hash, role:string
```

2. **生成文章脚手架**
```
生成 Post 脚手架（HTML 模式）：title:string, content:text, author_id:integer, published:boolean
```

3. **添加评论功能**
```
生成 Comment 脚手架（API 模式）：post_id:integer, user_id:integer, content:text
```

4. **添加标签系统**
```
创建 Tag 模型：name:string, slug:string
创建 PostTag 模型：post_id:integer, tag_id:integer
为 tags 生成 API 控制器：index, show, create, delete
```

### 构建 REST API

1. **产品目录**
```
生成 Product 脚手架（API）：name:string, description:text, price:decimal, stock:integer, sku:string
```

2. **订单系统**
```
生成 Order 脚手架（API）：user_id:integer, status:string, total:decimal, order_number:uuid
生成 OrderItem 脚手架（API）：order_id:integer, product_id:integer, quantity:integer, price:decimal
```

3. **购物车**
```
生成 CartItem 脚手架（API）：user_id:integer, product_id:integer, quantity:integer
```

## 调试技巧

### 查看生成的文件

```
帮我检查刚生成的 User 模型代码
```

Claude 可以使用 MCP 的文件读取功能查看生成的文件。

### 修改生成的代码

```
在 User 模型中添加一个 full_name() 方法
```

### 查看项目结构

```
显示当前项目的 models 和 controllers 目录结构
```

## 常见问题

**Q: 生成失败了怎么办？**
A: 确保：
- 在正确的 Loco 项目目录中（包含 `Cargo.toml`）
- 项目结构正确（有 `src/models`、`src/controllers` 等目录）
- 字段类型有效

**Q: 如何修改已生成的代码？**
A: 直接编辑生成的文件，或者重新运行生成命令（会覆盖现有文件）

**Q: 支持哪些数据库？**
A: Loco 支持 PostgreSQL、MySQL 和 SQLite，字段类型会自动转换为对应的数据库类型

**Q: 可以自定义模板吗？**
A: 当前版本使用 loco-gen 内置模板，后续版本会支持自定义模板

## 性能提示

- 批量操作：一次请求生成多个相关模型
- 明确指定字段类型，避免歧义
- 使用合适的脚手架类型（API vs HTML vs HTMX）

## 下一步

- 查看 [MCP 协议文档](https://modelcontextprotocol.io)
- 阅读 [Loco 框架文档](https://loco.rs)
- 探索更多 MCP 工具和集成

