# CLI Development Guide

## Server CLI Arguments

The cr8s server binary supports several CLI arguments for development and debugging:

```bash
cargo run --bin server -- --help
```

### Available Commands

| Flag | Description | Database Required |
|------|-------------|-------------------|
| `--help` | Show help message | ❌ No |
| `--version` | Show version | ❌ No |
| `--dump-state-traits` | Dump route-to-State<T> trait table and exit | ✅ Yes |
| `--check` | Enable CI mode: fail if manage()/State<T> mismatch found | ✅ Yes |
| `--output <PATH>` | Output route table to Markdown file | ✅ Yes |

## Integration Testing Workflow

### Test Structure

```
tests/
├── cli_integration.rs     # CLI command testing via docker compose
├── server_integration.rs  # HTTP API endpoint testing
└── (unit tests in src/tests/)
```

### Running All Tests

```bash
# Source the development environment
source scripts/dev-test-setup.sh

# Start services and run both CLI and server tests  
start-services && run-tests
```

### Individual Test Suites

```bash
# Test CLI commands
run-cli-tests

# Test HTTP API endpoints  
run-server-tests

# Run specific server test
run-single-server-test test_login_api

# Check server health and logs
check-server

# Utility aliases
dc ps                    # docker compose ps
dcr cli list-users      # docker compose run --rm cli list-users
```

### Development Environment Functions

The `dev-test-setup.sh` script provides these functions:

| Function | Purpose |
|----------|---------|
| `start-services` | Start postgres, redis, server + create test user |
| `stop-services` | Stop all services |
| `run-tests` | Run all integration tests (CLI + Server) |
| `run-cli-tests` | Run CLI integration tests |
| `run-server-tests` | Run server integration tests |
| `check-server` | Check server health and logs |
| `restart-server` | Restart just the server |

## Quick Development Workflow

For fastest iteration during development:

```bash
# One-time setup per terminal session
docker compose up -d postgres redis
export DATABASE_URL="postgres://postgres:secret@localhost:5432/cr8s"
export REDIS_URL="redis://127.0.0.1:6379/"

# Fast development iterations
cargo run --bin server
```

This hybrid approach gives you:
- Database and Redis running in containers (consistent, isolated)
- Server running natively (fast compilation, easy debugging)

## Testing CLI Features Locally

### Basic Commands (No Database Required)

These work immediately without any setup:

```bash
# Show help
cargo run --bin server -- --help

# Show version
cargo run --bin server -- --version
```

### Inspection Commands (Database Required)

For route inspection features, you need a running database and Redis instance:

#### Option 1: Use Docker Compose (Recommended)

```bash
# Start database and Redis
docker compose up -d postgres redis

# Use the inspection commands
docker compose run --rm server --dump-state-traits
docker compose run --rm server --check
docker compose run --rm server --output routes.md
```

#### Option 2: Hybrid Docker + Cargo

Run supporting services in Docker, but compile and run the server natively:

```bash
docker compose up -d postgres redis
export DATABASE_URL="postgres://postgres:secret@localhost:5432/cr8s"
export REDIS_URL="redis://127.0.0.1:6379/"
cargo run --bin server
```

**Important**: When running the server natively, use `127.0.0.1:6379` for Redis to connect from your host machine to the containerized Redis instance.

#### Option 3: Local Database Setup

If you prefer to run the database locally:

```bash
# 1. Start local Postgres and Redis
# (Instructions vary by OS - use brew, apt, etc.)

# 2. Set environment variables
export DATABASE_URL="postgres://postgres:secret@localhost:5432/cr8s"
export REDIS_URL="redis://127.0.0.1:6379/"

# 3. Create the database
createdb cr8s

# 4. Now inspection commands work
cargo run --bin server -- --dump-state-traits
cargo run --bin server -- --check
cargo run --bin server -- --output routes.md
```

## CI/Development Workflow

### Route State Analysis in CI

The `--check` flag is designed for CI environments:

```bash
# In CI, this will exit with code 1 if there are State<T> mismatches
docker compose run --rm server --check
```

### Generating Documentation

```bash
# Generate route documentation
docker compose run --rm server --output docs/routes.md
```

## Troubleshooting

### "DATABASE_URL must be set" Error

This error occurs when using inspection features without a database:

```
thread 'main' panicked at src/repository/database.rs:18:45:
DATABASE_URL must be set: NotPresent
```

**Solution:** Use Docker Compose or set up local database as shown above.

### "Pool not initialized" Error

If you see this error, it means the database connection failed:

```
Pool not initialized. Call init_pool_with_retry() first.
```

**Solutions:**
1. Check that Postgres is running and accessible
2. Verify `DATABASE_URL` points to the correct database
3. Ensure the database exists (`createdb cr8s` if using local setup)
4. Check network connectivity (especially in Docker)

### Redis Connection Issues

**"Redis not ready" or connection errors:**
- **Local development** (cargo run): Use `REDIS_URL="redis://127.0.0.1:6379/"`
- **Container deployment** (docker compose): Use `REDIS_URL="redis://redis/"`
- Ensure Redis container is running: `docker compose ps redis`

The different URLs are needed because:
- `127.0.0.1:6379` connects from host to container
- `redis` uses Docker's internal service name for container-to-container communication

See the [Container Usage Guide](container-usage-guide.md) for detailed Redis networking information.

## Development Best Practices

1. **Always test help/version flags** - These should work without any dependencies
2. **Use Docker for inspection features** - Avoids local database setup complexity  
3. **Run `--check` before commits** - Catches State<T> management issues early
4. **Update route docs** - Run `--output` when adding new routes
5. **Test integration endpoints** - Run `run-server-tests` to validate API changes

## Integration with Container Tests

The container test script validates all these scenarios automatically:

```bash
# Run the full test suite
bash scripts/container-test-script.sh
```

This tests both basic functionality and database-dependent features in a clean environment.
