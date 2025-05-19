# Docker usage tips

A handful of everyday commands for working with **cr8s** via Docker Compose.

---

## Start the API for manual testing using fully built containers

```bash
docker compose up --build -d  # app(server) + postgres + redis

# Tail the Rocket logs
docker compose logs -f app

# Basic health‑check
curl http://127.0.0.1:8000/health
```

---

## One‑liner: recreate DB & rerun tests

```bash
docker compose down       # stop any running containers
docker compose up -d postgres redis

# extract rust-dev version from Dockerfile
VERSION=$(awk '$1 == "FROM" && $2 ~ /rust-dev:/ { split($2, v, ":"); print v[2]; exit }' Dockerfile)
RUN="docker run --rm -v $PWD:$PWD -w $PWD -u $(id -u):$(id -g) ghcr.io/johnbasrai/cr8s/rust-dev:$VERSION"

# To run cargo tool chain
$RUN cargo [run, clean build, clippy, audit, outdated, deny, ] ...

# fresh migrations using diesel CLI
$RUN diesel setup
$RUN diesel migration run

# run tests using native cargo (optional: use docker run cr8s-test-runner)
cargo test --workspace --all-targets --all-features
```
> Note to creat the test containers `cr8s-test-runner` and `cr8s-test-seeder` you will need to run `scripts/dev/build-images.sh`

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
