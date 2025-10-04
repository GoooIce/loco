# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Loco is a Rust web framework inspired by Ruby on Rails, providing a "one-person framework" for building web applications. It emphasizes convention over configuration, rapid development, and includes built-in features for ORM integration, controllers, views, background jobs, scheduling, mailers, storage, and caching.

## Development Commands

### Building and Running
- `cargo build` - Build the framework library
- `cargo test` - Run tests (use `cargo test --features testing` for testing features)
- `cargo loco start` - Start a Loco application (from within a Loco app)
- `cargo install loco` - Install the Loco CLI
- `cargo install sea-orm-cli` - Install Sea ORM CLI (when using databases)

### Framework Development
- `cargo run --package xtask -- <command>` - Run xtask commands for framework development
- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo doc` - Generate documentation

### Application Development (when using Loco to build apps)
- `loco new` - Create a new Loco application
- `cargo loco start` - Start the development server
- `cargo loco db <command>` - Database operations (migrate, seed, etc.)
- `cargo loco routes` - List all application endpoints
- `cargo loco middleware` - List middleware configurations

## Architecture Overview

### Core Framework Structure
The framework is organized into several key modules:

- **App & Boot (`src/app.rs`, `src/boot.rs`)**: Core application lifecycle, startup, and shutdown management
- **Controllers (`src/controller/`)**: Web request handling, routing, middleware, and response formatting
- **Models (`src/model/`)**: Database models and query builders (when `with-db` feature is enabled)
- **Background Workers (`src/bgworker/`)**: Support for Redis, PostgreSQL, and SQLite-based job queues
- **Storage (`src/storage/`)**: File storage abstraction supporting local, memory, and cloud providers (AWS S3, GCP, Azure)
- **Cache (`src/cache/`)**: Caching abstraction with in-memory and Redis support
- **Mailers (`src/mailer/`)**: Email sending capabilities using Tera templates
- **Scheduler (`src/scheduler/`)**: Cron-like task scheduling
- **Auth (`src/auth/`)**: JWT-based authentication (when `auth_jwt` feature is enabled)
- **Testing (`src/testing/`)**: Testing utilities and helpers (when `testing` feature is enabled)

### Feature Flags
The framework uses extensive feature flags for modularity:
- `with-db`: Database functionality with Sea ORM
- `auth_jwt`: JWT authentication
- `cli`: Command-line interface
- `testing`: Testing utilities
- `cache_inmem`/`cache_redis`: Cache implementations
- `bg_redis`/`bg_pg`/`bg_sqlt`: Background worker implementations
- `storage_*`: Various storage providers
- `embedded_assets`: Asset embedding support

### Application Structure
Loco applications follow a Rails-like convention:
- Implement the `Hooks` trait for application lifecycle
- Use `AppRoutes` for defining application routes
- Controllers handle web requests with automatic JSON/form data extraction
- Models use Sea ORM for database operations
- Background workers implement the `Worker` trait with a `perform` method
- Views use Tera templating engine

### Key Components

#### SharedStore
Type-safe heterogeneous storage for application-wide data sharing between components.

#### Configuration
Configuration management with environment-based settings and YAML/TOML support.

#### Middleware System
Composable middleware stack including:
- Request logging and tracing
- CORS handling
- Compression
- Security headers
- Static asset serving
- Request ID generation
- Timeout handling

#### Error Handling
Centralized error handling with detailed error responses and error logging.

## Development Notes

- Uses Rust 2021 edition, minimum Rust 1.70
- Built on Axum web framework for performance and extensibility
- Sea ORM for database operations with SQLite and PostgreSQL support
- Tokio async runtime throughout
- Extensive use of `async-trait` for extensible interfaces
- Comprehensive testing support with test utilities and fixtures

## Testing

Run tests with:
```bash
cargo test
cargo test --features testing
```

For integration tests:
```bash
cargo test --features integration_test
```
