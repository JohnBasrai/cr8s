# Docker usage tips

A handful of everyday commands for working with **cr8s** via Docker Compose.

---

## Start the API for manual testing

```bash
# Build image & launch full stack (detached)
docker compose up --build -d        # app + postgres + redis

# Tail the Rocket logs
docker compose logs -f app

# Basic health‑check
curl http://127.0.0.1:8000/health
```

---

## One‑liner: recreate DB & rerun tests

```bash
docker compose down        # stop any running services
docker compose up -d postgres redis
# fresh migrations
docker compose run --rm app diesel setup
# run tests in isolated container
docker compose run --rm --service-ports app \
  bash -c 'cargo run --bin server & sleep 3 && cargo test -- --test-threads=1'
```

---

## Database maintenance cheatsheet

| Task               | Command                                                |
| ------------------ | ------------------------------------------------------ |
| Re‑run migrations  | `docker compose run --rm app diesel migration run`     |
| Drop & recreate DB | `docker compose run --rm app diesel database reset`    |
| psql shell         | `docker compose exec postgres psql -U postgres app_db` |
| List tables        | `\dt` inside psql                                      |

---

## Clean up everything

```bash
docker compose down -v   # stops containers and removes named volumes
```

> Remove dangling images / build cache with `docker system prune` (optional).
