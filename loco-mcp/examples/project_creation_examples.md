# Loco Project Creation Examples

This document provides examples of using the `loco_create_project` tool to create different types of Loco projects.

## Basic Usage

### Creating a SaaS Application

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "task_manager",
    "template_type": "saas",
    "destination_path": "/Users/username/projects/task_manager",
    "database_type": "postgres",
    "background_worker": "redis",
    "asset_serving": true
  }
}
```

**Expected Result:**
```
✅ 生成成功！

Created saas project 'task_manager' at '/Users/username/projects/task_manager'
Database: postgres
Background worker: redis
Asset serving: local
```

### Creating a REST API

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "user_service_api",
    "template_type": "rest_api",
    "destination_path": "/Users/username/projects/user_service_api",
    "database_type": "postgres"
  }
}
```

**Expected Result:**
```
✅ 生成成功！

Created rest_api project 'user_service_api' at '/Users/username/projects/user_service_api'
Database: postgres
Background worker: none
Asset serving: none
```

### Creating a Lightweight Service

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "health_checker",
    "template_type": "lightweight",
    "destination_path": "/Users/username/projects/health_checker"
  }
}
```

**Expected Result:**
```
✅ 生成成功！

Created lightweight project 'health_checker' at '/Users/username/projects/health_checker'
Database: sqlite
Background worker: none
Asset serving: none
```

## Advanced Examples

### Custom Configuration

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "analytics_platform",
    "template_type": "saas",
    "destination_path": "/Users/username/projects/analytics_platform",
    "database_type": "postgres",
    "background_worker": "redis",
    "asset_serving": true
  }
}
```

### SQLite-based SaaS Application

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "simple_blog",
    "template_type": "saas",
    "destination_path": "/Users/username/projects/simple_blog",
    "database_type": "sqlite",
    "background_worker": "none",
    "asset_serving": false
  }
}
```

### API with Asset Serving

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "content_api",
    "template_type": "rest_api",
    "destination_path": "/Users/username/projects/content_api",
    "database_type": "postgres",
    "background_worker": "sqlite",
    "asset_serving": true
  }
}
```

## Error Handling Examples

### Invalid Project Name

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "My-App",
    "template_type": "saas",
    "destination_path": "/Users/username/projects/My-App"
  }
}
```

**Expected Error:**
```
❌ 错误：Invalid project name 'My-App'. Must start with lowercase letter and contain only lowercase letters, numbers, and underscores
```

### Invalid Template Type

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "my_app",
    "template_type": "invalid_template",
    "destination_path": "/Users/username/projects/my_app"
  }
}
```

**Expected Error:**
```
❌ 错误：Invalid template_type 'invalid_template'. Must be one of: saas, rest_api, lightweight
```

### Directory Already Exists

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "existing_project",
    "template_type": "lightweight",
    "destination_path": "/Users/username/projects/existing_project"
  }
}
```

**Expected Error:**
```
❌ 错误：Destination path '/Users/username/projects/existing_project' already exists
```

## Best Practices

### Project Naming

- Use snake_case: `user_service`, `task_manager`, `blog_platform`
- Start with lowercase letter
- Use underscores instead of spaces or hyphens
- Be descriptive but concise

### Template Selection

- **SaaS**: Full-featured web applications with authentication, background jobs, and asset serving
- **REST API**: API-only applications, microservices, or backend services
- **Lightweight**: Simple services, utilities, or minimal applications

### Database Configuration

- **PostgreSQL**: Production applications, complex queries, high concurrency
- **SQLite**: Development, simple applications, single-user tools
- **None**: Stateless services, simple utilities, or when using external databases

### Background Workers

- **Redis**: Production applications with background jobs, caching, or pub/sub
- **PostgreSQL**: Applications using database-based job queues
- **SQLite**: Simple applications with basic job processing
- **None**: Applications without background processing needs

## Complete Workflow Example

### 1. Create a SaaS Project

```json
{
  "tool": "loco_create_project",
  "arguments": {
    "project_name": "crm_system",
    "template_type": "saas",
    "destination_path": "/Users/username/projects/crm_system"
  }
}
```

### 2. Generate a User Model

```json
{
  "tool": "loco_generate_model",
  "arguments": {
    "project_path": "/Users/username/projects/crm_system",
    "name": "user",
    "fields": {
      "email": "string",
      "first_name": "string",
      "last_name": "string",
      "phone": "string",
      "company": "string"
    },
    "with_timestamps": true
  }
}
```

### 3. Generate User Scaffold

```json
{
  "tool": "loco_generate_scaffold",
  "arguments": {
    "project_path": "/Users/username/projects/crm_system",
    "name": "user",
    "fields": {
      "email": "string",
      "first_name": "string",
      "last_name": "string",
      "phone": "string",
      "company": "string"
    },
    "kind": "html",
    "with_timestamps": true
  }
}
```

### 4. Run Database Migration

```json
{
  "tool": "migrate_db",
  "arguments": {
    "project_path": "/Users/username/projects/crm_system",
    "approvals": ["ops_lead", "security_officer"],
    "environment": "development",
    "timeout_seconds": 60,
    "dependencies": ["postgres", "redis"]
  }
}
```

## Project Structure

### SaaS Template Structure
```
my_saas_app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── models/
│   │   └── mod.rs
│   ├── controllers/
│   │   ├── mod.rs
│   │   └── health.rs
│   └── views/
│       └── mod.rs
└── tests/
    └── mod.rs
```

### REST API Template Structure
```
my_api_app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── models/
│   │   └── mod.rs
│   └── controllers/
│       ├── mod.rs
│       └── health.rs
└── tests/
    └── mod.rs
```

### Lightweight Template Structure
```
my_lightweight_app/
├── Cargo.toml
├── src/
│   └── main.rs
└── tests/
    └── mod.rs
```

## Next Steps

After creating a project:

1. **Review the generated code**: Check the files and understand the structure
2. **Customize configuration**: Modify `Cargo.toml` and other config files
3. **Add your business logic**: Implement models, controllers, and views
4. **Run tests**: Ensure everything works as expected
5. **Deploy**: Follow Loco deployment guides for production

## Troubleshooting

### Common Issues

1. **Permission Denied**: Ensure the destination directory is writable
2. **Invalid Paths**: Use absolute paths for destination directories
3. **Missing Dependencies**: Ensure Rust and Python are properly installed
4. **Compilation Errors**: Check that the loco-bindings are built correctly

### Getting Help

- Check the [Loco documentation](https://loco.rs/docs)
- Review the [API reference](../docs/API.md)
- Open an issue on [GitHub](https://github.com/loco-rs/loco/issues)