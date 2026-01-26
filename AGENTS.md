# AGENTS.md - Axum Backend Project

## Project Overview

Rust web backend with Axum v0.8.8, PostgreSQL, and REST API for blog post management. Uses Rust edition 2024.

## Build, Lint, and Test Commands

```bash
# Build and run
cargo build
cargo build --release
cargo run

# Testing
cargo test                          # all tests
cargo test test_name                # single test (exact match)
cargo test test_pattern             # pattern match
cargo test -- --nocapture           # with output

# Linting and formatting
cargo clippy                        # linter
cargo clippy --fix                  # auto-apply fixes
cargo check                         # check without building
cargo fmt                           # format
cargo fmt --check                   # check formatting
```

## Code Style Guidelines

### General
- Write idiomatic Rust 2024 edition code
- Prefer explicit error handling over unwrap/panic
- Use async/await for all I/O operations
- Use `Arc<Client>` for shared database state

### Imports and Modules
- Organize: `models/`, `handlers/`, `services/`, `database.rs`
- Each module has `mod.rs` exporting submodules
- Use `crate::` for absolute imports
- Group imports: std, external, internal

### Naming Conventions
- **Files**: snake_case (`post_handler.rs`)
- **Structs**: PascalCase (`Post`, `ApiResponse`)
- **Functions**: snake_case (`get_all_posts`)
- **Variables**: snake_case (`db_conn`, `post_id`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Types**: Explicit in function signatures

### Formatting
- Default Rustfmt settings (4 spaces, 100 char width)
- Opening braces on same line
- Trailing commas in multi-line expressions

### Error Handling
- Return `Result<T, tokio_postgres::Error>` for DB operations
- Use `?` for error propagation
- Define `AppError` enum: Database, Pool, NotFound, BadRequest, InternalServerError
- Implement `IntoResponse` returning `success: false` JSON
- Handle `deadpool_postgres::PoolError` separately
- Log with `tracing::error!` for background errors

### Logging and Tracing
- Initialize with `tracing_subscriber::registry()` and EnvFilter
- Default level: info (tower_http: info, axum::rejection: trace)
- Use `tracing::info!`, `tracing::error!` macros

### Async and Concurrency
- Use `tokio-postgres` with deadpool connection pooling
- `DbPool` (deadpool::Pool) in Axum state
- `State<DbPool>` for DI in handlers
- Get client: `pool.get().await`

### Types and Serialization
- `serde::{Serialize, Deserialize}` for all serializable types
- `Json<T>` from axum for responses
- `uuid::Uuid` for IDs, `chrono::DateTime<Utc>` for timestamps
- Clone derives acceptable for simple types

### Axum Framework Patterns
- Routes under `/v1` prefix
- `Query<T>` for query params, `Json<T>` for request bodies
- CORS: `CorsLayer::permissive()`
- Trace logging: `TraceLayer::new_for_http()`
- Merge routers with `.merge(sub_router)`

### API Response Patterns
- `ApiResponse<T>` wrapper: `success: bool`, `data: T`, `error: Option<String>`
- Success: `Json(ApiResponse::success(data))`
- Error: `AppError` with `IntoResponse`

### Database
- Parameterized queries: `$1`, `$2`
- Use `JOIN` for related data
- Return `Result<Vec<T>, tokio_postgres::Error>` from services

### Security
- Use `.env` files with `dotenvy` for secrets
- Validate all query parameters
- Parameterized queries prevent SQL injection

### Testing
- Tests in `#[cfg(test)]` module in same file
- Mock database connections for unit tests

### Git Workflow
- Imperative commit messages
- No force pushes to main
- Run `cargo clippy` and `cargo fmt` before committing
- Never commit: `.env`, `database.db`, `target/`

## Key Files

| Component | Location |
|-----------|----------|
| Entry point | `src/main.rs` |
| Database | `src/database.rs` |
| Error handling | `src/error.rs` |
| API response | `src/response.rs` |
| Config | `src/config.rs`, `.env` |
| Models | `src/models/{post,user,tag}.rs` |
| Handlers | `src/handlers/{health,post,tag}.rs` |
| Services | `src/services/{post,tag}.rs` |
