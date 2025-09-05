# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Testing
- `cargo test` - Run all tests for the Loco framework
- `cargo xtask test` - Run comprehensive tests across all Loco resources (lib, cli, starters, examples)
- `cargo xtask test --quick` - Run only Loco library tests
- `cargo loco doctor` - Validate and diagnose configurations
- `cargo loco doctor --environment test` - Check test environment requirements

### Loco CLI Commands
- `cargo loco start` - Start the Loco application server
- `cargo loco start --worker` - Start standalone worker process
- `cargo loco start --server-and-worker` - Start both server and worker
- `cargo loco start --worker <tag>` - Start worker for specific job tags
- `cargo loco generate <component>` - Generate code components (debug builds only)
- `cargo loco task <name>` - Run custom tasks
- `cargo loco scheduler` - Run the scheduler
- `cargo loco db reset` - Reset and migrate database
- `cargo loco db entities` - Generate database entities

### Code Quality
- `cargo fmt` - Format code
- `cargo clippy` - Run linter with pedantic checks
- `cargo clippy -- -W clippy::pedantic -W rust-2021-compatibility -W rust-2018-idioms`

## Architecture Overview

Loco is a Rust web framework inspired by Ruby on Rails, following convention over configuration principles. The framework is built on Axum and provides:

### Core Components
- **App Structure**: Central `App` struct implementing `Hooks` trait for lifecycle management
- **Controllers**: Handle HTTP requests with validation, formatting, and middleware support
- **Models**: SeaORM-based database entities with relationships and validation
- **Views**: Tera templating engine for dynamic HTML generation
- **Background Workers**: Redis, PostgreSQL, or SQLite-backed job queues
- **Scheduler**: Cron-like task scheduling with English-to-cron conversion
- **Mailers**: Background email sending with template support
- **Storage**: Abstracted file storage (local, AWS S3, GCP, Azure)
- **Cache**: In-memory or Redis caching layer
- **DDD Integration**: Domain-Driven Design patterns with Entity, AggregateRoot, Repository traits

### Key Architectural Patterns
- **Boot System**: `boot.rs` handles application startup with different modes (ServerOnly, ServerAndWorker, WorkerOnly, All)
- **Shared Store**: Type-safe heterogeneous storage for application data using `DashMap`
- **Middleware**: Tower-based middleware stack for logging, CORS, security headers, etc.
- **Configuration**: Environment-based configuration with YAML files
- **Testing**: Integrated testing with `axum-test` and snapshot testing with `insta`
- **Async Patterns**: Comprehensive async/await support using `#[async_trait]` macro

### Feature Flags
- `auth_jwt` - JWT authentication support
- `with-db` - Database functionality (SeaORM)
- `cli` - Command-line interface
- `testing` - Testing utilities
- `bg_redis`/`bg_pg`/`bg_sqlt` - Background worker backends
- `storage_*` - Cloud storage providers
- `cache_*` - Cache backends
- `mcp` - Model Context Protocol server support
- `i18n` - Internationalization support (Chinese, Japanese, English)

### Project Structure
- `src/` - Core framework code
- `loco-gen/` - Code generation utilities
- `loco-new/` - Application template generator
- `loco-cli/` - CLI tool (deprecated, use `loco-new`)
- `xtask/` - Development task runner
- `tests/` - Framework tests
- `examples/` - Example applications
- `docs-site/` - Documentation website
- `session_memory/` - Cross-session development context persistence

### Development Environment Requirements
- Redis server running (for background jobs)
- Database (PostgreSQL/SQLite) for DB-dependent features
- Node.js/pnpm for frontend builds in starter templates

### Testing Strategy
- Unit tests for individual components
- Integration tests with testcontainers
- Snapshot testing for code generation
- CI tests across multiple configurations
- Starter applications tested against new versions

### Code Generation
Loco provides code generation for:
- Models with database migrations
- Controllers with RESTful routes
- Background workers
- Mailers with email templates
- Scaffolding for complete CRUD operations
- Deployment configurations

### Background Processing
Supports multiple worker backends:
- Redis (recommended for production)
- PostgreSQL (using SKIP LOCKED)
- SQLite (using application locks)

### Database Integration
- SeaORM for database abstraction
- Automatic migration generation
- Entity generation from database schema
- Support for PostgreSQL and SQLite

### Asset Management
- Static file serving
- Embedded assets option
- Frontend build integration (React, Vue, etc.)

### Domain-Driven Design Integration
The framework includes comprehensive DDD support:
- **Entity Trait**: Base trait for domain entities with equality semantics
- **AggregateRoot Trait**: Entities with event sourcing capabilities
- **Repository Trait**: Generic data access abstraction
- **ValueObject Trait**: Immutable value objects
- **DomainError**: Comprehensive error hierarchy for domain-specific errors
- **Result Types**: Consistent `Result<T, DomainError>` pattern throughout

### MCP (Model Context Protocol) Support
- Built-in MCP server for tool integration
- Protocol handlers for request/response management
- Tool registry for custom tool registration
- HTTP and WebSocket transport support
- Integration with existing Loco application context

### Internationalization (i18n)
- Multi-language support (Chinese, Japanese, English)
- Language detection from headers, URL parameters, and cookies
- Template integration with Tera
- Fallback language support
- Translation file management

### Error Handling Patterns
- Centralized error handling with `thiserror`
- Context-aware error messages
- Automatic error conversions using `From` traits
- Domain-specific error hierarchy for DDD patterns
- User-friendly error messages with actionable guidance

### Async Patterns
- Extensive use of `#[async_trait]` for trait implementations
- Consistent async/await patterns throughout
- Proper lifecycle management for async resources
- Integration with Tokio runtime

@docs/rust_compilation_best_practices.md