# Axum Backend

A simple Rust web backend built with Axum and PostgreSQL for blog post management.

## Features

- REST API for blog posts
- PostgreSQL database with connection pooling
- Input validation
- Structured logging
- Docker support

## Quick Start

### 1. Set up environment

```bash
cp .env.example .env
```

Edit `.env` with your database credentials:

```env
PORT=8080
DATABASE_URL="postgresql://postgres:password@localhost:5432/axumbackend"
DB_POOL_MAX_SIZE=20
```

### 2. Run the application

```bash
cargo run
```

Server starts on `http://localhost:8080`

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/health` | Health check |
| GET | `/v1/posts` | Get all posts |
| GET | `/v1/posts/random?limit=N` | Get random posts |
| GET | `/v1/posts/tag/{tag}` | Get posts by tag |
| GET | `/v1/posts/u/{username}/{slug}` | Get post by author |

## Development

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Docker

```bash
docker build -t axumbackend .
docker run -p 8080:8080 --env-file .env axumbackend
```

## Project Structure

```
src/
├── main.rs         # Entry point
├── config.rs       # Configuration
├── database.rs     # Database setup
├── error.rs        # Error handling
├── response.rs     # API responses
├── models/         # Data models
├── handlers/       # HTTP handlers
└── services/       # Business logic
```

## Tech Stack

- **Framework**: Axum 0.8
- **Database**: PostgreSQL + deadpool-postgres
- **Async**: Tokio
- **Serialization**: Serde
- **Validation**: axum-valid
