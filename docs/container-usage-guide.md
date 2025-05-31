# Container Usage Guide (v2)

Essential commands for using **cr8s** with Docker Compose.

---

## Quick Start

```bash
# Start the full stack
docker compose up -d

# Test the API
curl http://127.0.0.1:8000/health

# View logs (in another terminal)
docker logs -f cr8s-server-1
```

---

## Environment Variables

The server container requires these environment variables in docker-compose.yml:

```yaml
server:
  environment:
    ROCKET_PROFILE: default
    DATABASE_URL: postgres://postgres:secret@postgres:5432/cr8s
    REDIS_URL: redis://redis/  # Required for container-to-container communication
```

**Redis URL Notes:**
- Use `redis://redis/` for container-to-container communication
- The service name `redis` is resolved by Docker's internal DNS
- This differs from local development which uses `redis://127.0.0.1:6379/`

---

## Container Networking

### Redis Connection Differences

The Redis URL varies depending on your deployment:

| Environment | Redis URL | Why |
|-------------|-----------|-----|
| Full Docker Compose | `redis://redis/` | Container-to-container via service name |
| Local Development | `redis://127.0.0.1:6379/` | Host-to-container via localhost |

Docker Compose creates an internal network where services can reach each other by name (`redis`, `postgres`), but when running locally with `cargo run`, you connect through the exposed ports on localhost.

---

## Container Management

```bash
# Update to latest images
docker compose down
docker compose pull  
docker compose up -d

# Stop everything
docker compose down

# Clean up (removes data!)
docker compose down -v
```

---

## CLI Operations

```bash
# Initialize system (first time)
docker compose run --rm server ./cli init-default-roles

# User management
docker compose run --rm server ./cli create-user --username admin --roles admin
docker compose run --rm server ./cli list-users

# Get help
docker compose run --rm server ./cli --help
```

---

## Database Access

```bash
# PostgreSQL shell
docker compose exec postgres psql -U postgres cr8s

# Redis CLI
docker compose exec redis redis-cli
```

---

## Troubleshooting

### Redis Connection Failures

**Symptoms:** "Redis not ready" messages or connection timeouts

**Solutions:**
1. Verify Redis container is running: `docker compose ps redis`
2. Check Redis URL matches your environment:
   - Container mode: `REDIS_URL=redis://redis/`
   - Local mode: `REDIS_URL=redis://127.0.0.1:6379/`
3. Restart Redis if needed: `docker compose restart redis`

### Service Discovery Issues

If containers can't find each other, verify:
- All services are in the same docker-compose.yml
- Service names match the URLs (e.g., `redis://redis/` expects service named `redis`)
- No port conflicts with host system