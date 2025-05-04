# Native workflow (community-supported)

> **Heads-up 🚧** The maintainer develops & tests with Docker only. These steps  
> *should* work on any Linux / macOS / WSL host provided PostgreSQL client  
> headers are present, but they are **not covered by CI**. If you refine them,  
> please open a PR!

---

## 1  Install prerequisites

Ubuntu example:

```bash
sudo apt-get update && sudo apt-get install -y libpq-dev pkg-config
````

Install **Diesel CLI** once (provides `diesel setup`):

```bash
cargo install diesel_cli --no-default-features --features postgres
```

---

## 2  Launch services

Run Postgres & Redis however you like (native packages, Homebrew, Docker). Quick Docker option:

```bash
# Postgres 16 on host port 5432
docker run -d --name pg -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:16

# Redis on host port 6379
docker run -d --name redis -p 6379:6379 redis:7
```

---

## 3  Set environment variables

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/app_db
export ROCKET_DATABASES='{
  postgres={url=postgres://postgres:postgres@localhost:5432/app_db},
  redis={url=redis://localhost:6379}
}'
```

---

## 4  Create database and run migrations

```bash
diesel setup      # creates DB + runs SQL in migrations/
```

---

## 5  Run the test-suite

```bash
cargo test --all-features -- --test-threads=1
```

This spawns an in-process Rocket server on **127.0.0.1:8000** and the tests hit it via `reqwest`.

---

## Troubleshooting

| Symptom                       | Fix                                                                             |
| ----------------------------- | ------------------------------------------------------------------------------- |
| `diesel: command not found`   | Ensure `~/.cargo/bin` is in `$PATH`, or open a new shell after `cargo install`. |
| `could not connect to server` | Verify Postgres is listening on **5432** and `DATABASE_URL` is correct.         |
| Migration errors              | Confirm `diesel setup` completed without errors; check `migrations/`.           |
| `REQWEST connect error`       | Make sure Rocket server is running and Redis is reachable on **6379**.          |

---

## Contributing

If you automate these steps (systemd unit, Homebrew formula, Windows PowerShell, etc.) please submit a pull request – all improvements are welcome!

