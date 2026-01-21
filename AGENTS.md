# AGENTS.md - Rocket Backend Project

## Project Overview

This is a Rust web backend application built with the Rocket web framework (v0.5.1) using PostgreSQL. The project implements a blog post management system with REST API endpoints. Uses Rust edition 2024.

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

### Imports and Module Structure
- Organize code into modules: `models/`, `routes/`, `services/`, `database.rs`
- Each module has a `mod.rs` that exports submodules
- Use absolute imports with `crate::` for internal modules
- Group imports by crate (std, external, internal)
- Use `#[macro_use]` for Rocket macros at file top when needed

### Naming Conventions
- **Files**: snake_case (e.g., `post_service.rs`, `health_check.rs`)
- **Structs**: PascalCase (e.g., `Post`, `User`)
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
- Return `Result<T, Error>` for functions that can fail
- Use `?` operator for propagating errors in async contexts
- Use `unwrap_or_else` or `unwrap_or` for fallible operations with defaults
- Log errors with `eprintln!` for connection/background errors
- Never use `expect()` in production route handlers

### Async and Concurrency
- All database operations are async using tokio-postgres
- Use `tokio::spawn` for background connection handling
- Wrap database client in `Arc<State>` for Rocket managed state
- Use `rocket::State<Arc<Client>>` for DI in route handlers

### Types and Serialization
- Use `serde::{Serialize, Deserialize}` for all serializable types
- Use `Json<Vec<T>>` for JSON response types in routes
- Use `uuid::Uuid` for unique identifiers
- Use `chrono::DateTime<Utc>` for timestamps
- Clone derives are acceptable for simple data types

### Rocket Framework Patterns
- Mount routes under `/v1` prefix: `.mount("/v1", routes![...])`
- Use attribute macros for route definitions: `#[get("/path")]`
- State management: `rocket.manage(Arc::new(client))`
- Health check endpoint at `GET /health` returning `"OK"`

### Database Queries
- Use parameterized queries with `$1`, `$2` placeholders
- Use `JOIN` statements for related data (posts with users)
- Return `Result<Vec<T>, tokio_postgres::Error>` from service functions
- Handle connection errors in background spawn

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
- **Models**: `src/models/{post,user}.rs`
- **Routes**: `src/routes/{health,post}.rs`
- **Services**: `src/services/post.rs`
- **Config**: `Cargo.toml`, `.env`

## Dependencies

- `rocket` (v0.5.1) - Web framework with JSON support
- `tokio-postgres` (v0.7.12) - PostgreSQL async client
- `serde` (v1.0.228) - Serialization framework
- `uuid` (v1.11.0) - UUID generation
- `chrono` (v0.4.43) - DateTime handling
- `dotenvy` (v0.15) - Environment variable loading
