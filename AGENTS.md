# AGENTS.md - Axum Backend Project

## Project Overview

This is a Rust web backend application built with the Axum web framework (v0.7.5) using PostgreSQL. The project implements a blog post management system with REST API endpoints. Uses Rust edition 2024.

## Build, Lint, and Test Commands

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the application
cargo run

# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run clippy linter
cargo clippy

# Run clippy with fixes (auto-apply)
cargo clippy --fix

# Check code without building
cargo check

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

## Code Style Guidelines

### General Principles
- Write clean, idiomatic Rust code following the 2024 edition conventions
- Prefer explicit error handling over unwrap/panic in production code
- Use async/await for all I/O operations (database, HTTP)
- Use `Arc<Client>` for shared database connection state

### Imports and Module Structure
- Organize code into modules: `models/`, `handlers/`, `services/`, `database.rs`
- Each module has a `mod.rs` that exports submodules
- Use absolute imports with `crate::` for internal modules
- Group imports by crate (std, external, internal)
- Use `use` statements at top level, not `#[macro_use]`

### Naming Conventions
- **Files**: snake_case (e.g., `post_handler.rs`, `health_check.rs`)
- **Structs**: PascalCase (e.g., `Post`, `User`, `ApiResponse`)
- **Functions**: snake_case (e.g., `get_all_posts`, `connect`)
- **Variables**: snake_case (e.g., `db_conn`, `post_id`)
- **Constants**: SCREAMING_SNAKE_CASE for global constants
- **Modules**: snake_case
- **Types in function signatures**: Use explicit types, avoid inference

### Formatting and Style
- Use default Rustfmt settings (no custom config)
- Maximum line length: 100 characters (default)
- Use 4 spaces for indentation
- Place opening braces on same line as declaration
- Add trailing comma in multi-line expressions

### Error Handling
- Return `Result<T, tokio_postgres::Error>` for database operations
- Use `?` operator for propagating errors in async contexts
- Use `unwrap_or_else` or `unwrap_or` for fallible operations with defaults
- Log errors with `eprintln!` for connection/background errors
- Handle errors gracefully in handlers with fallbacks to empty collections

### Async and Concurrency
- All database operations are async using tokio-postgres
- Use `tokio::spawn` for background connection handling
- Wrap database client in `Arc<Client>` for Axum state management
- Use `State<Arc<Client>>` for DI in route handlers

### Types and Serialization
- Use `serde::{Serialize, Deserialize}` for all serializable types
- Use `Json<T>` from axum for JSON response types in routes
- Use `uuid::Uuid` for unique identifiers
- Use `chrono::DateTime<Utc>` for timestamps
- Clone derives are acceptable for simple data types

### Axum Framework Patterns
- Mount routes under `/v1` prefix: `Router::new().route("/v1/posts", get(handler))`
- Use `routing::get/post` extractors for defining routes
- State management: `Router::with_state(Arc::new(client))`
- Health check endpoint at `GET /health` returning `"OK"`
- Use `Query<T>` for query parameters, `Json<T>` for request bodies
- Add CORS with `tower_http::cors::CorsLayer::permissive()`

### Database Queries
- Use parameterized queries with `$1`, `$2` placeholders
- Use `JOIN` statements for related data (posts with users)
- Return `Result<Vec<T>, tokio_postgres::Error>` from service functions
- Handle connection errors in background spawn with `tokio::spawn`
- Handle truncation logic in services (e.g., body to 200 chars)

### Security
- Never commit secrets; use `.env` files with `dotenvy`
- DATABASE_URL is loaded from environment with fallback defaults
- Validate all query parameters (e.g., `limit: Option<i64>`)
- Use parameterized queries to prevent SQL injection

### Testing
- Place tests in same file using `#[cfg(test)]` module
- Use `#[test]` attribute for test functions
- Run single tests with `cargo test function_name`
- Mock database connections for unit tests

### Git Workflow
- Commit messages should be concise, imperative mood
- No force pushes to main without explicit approval
- Run `cargo clippy` and `cargo fmt` before committing
- Never commit generated files (database.db, target/)

## Key File Locations

- **Entry point**: `src/main.rs`
- **Database**: `src/database.rs`
- **Models**: `src/models/{post,user,response}.rs`
- **Handlers**: `src/handlers/{health,post,tag}.rs`
- **Services**: `src/services/{post,tag}.rs`
- **Config**: `src/config.rs`, `.env`

## Dependencies

- `axum` (v0.7.5) - Web framework with JSON support
- `axum-extra` (v0.9.3) - Additional Axum utilities
- `tower-http` (v0.5.2) - HTTP middleware including CORS
- `tokio-postgres` (v0.7.12) - PostgreSQL async client
- `postgres-types` (v0.2.9) - PostgreSQL type support
- `serde` (v1.0.228) - Serialization framework
- `serde_json` (v1.0.149) - JSON serialization
- `uuid` (v1.11.0) - UUID generation
- `chrono` (v0.4.43) - DateTime handling
- `dotenvy` (v0.15) - Environment variable loading
