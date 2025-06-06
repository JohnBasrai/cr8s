# CR8S – Database Schema

## Tables and Schema

### `app_user`
- System login identity
- Fields: `id`, `username`, `password`, `created_at`, `author_id`
- `author_id` is nullable and references `author(id)`

### `author`
- Represents a Rust contributor or crate author
- Can exist without an `app_user`
- Fields: `id`, `name`, `email`, `created_at`

### `crate` (represented as `crate_` in `schema.rs`)
- Represents a Rust crate/project
- Fields: `id`, `author_id`, `code`, `name`, `version`, `description`, `created_at`
- Foreign key: `author_id` → `author.id`

> #### `crate` Table Fields
> 
> | Column        | Type           | Purpose / Usage                                        |
> | :------------ | :------------- | :----------------------------------------------------- |
> | `id`          | `SERIAL`       | Primary key (internal use only)                        |
> | `author_id`   | `integer`      | FK to `author(id)`                                     |
> | `code`        | `varchar(64)`  | Internal identifier / unique slug (e.g. `actix-web`)   |
> | `name`        | `varchar(128)` | Human-readable label (`"Actix Web Framework"`)         |
> | `version`     | `varchar(64)`  | Optional – unless you're tracking releases (`"1.2.3"`) |
> | `description` | `text`         | Often empty, long, or copy-pasted boilerplate          |
> | `created_at`  | `timestamp`    | Fine to keep; may not be shown unless needed           |
> 

### `role`
- Static lookup table for system roles
- Fields: `id`, `code`, `name`, `created_at`

### `user_role`
- Join table between `app_user` and `role`
- Fields: `id`, `user_id`, `role_id`
- Uniqueness constraint on `(user_id, role_id)`

---

## Trait Naming Convention

Each table is abstracted via a trait:

| Table           | Trait Interface      | Notes                                             |
|-----------------|----------------------|---------------------------------------------------|
| `app_user`      | `AppUserTableTrait`  |                                                   |
| `author`        | `AuthorTableTrait`   |                                                   |
| `crate`         | `CrateTableTrait`    |                                                   |
| `role`          | `RoleTableTrait`     |                                                   |
| `user_role`     | `UserRoleTableTrait` |                                                   |
| *n/a (admin)*   | `ServerInfoTrait`    | Not tied to a table; diagnostic utility           |
| *n/a (infra)*   | `DBContextTrait`     | Abstract lifecycle hook for global DB state       |


- Traits live in `src/domain/`
- Each trait is defined in its own file and all public symbols are reexported on `mod.rs`

---

## Rust Module Structure

### `src/domain/`
- Pure traits, one per file
- No awareness of SQLx or mocks

```
src/domain/
├── app_user.rs        # DMO + AppUserTableTrait
├── authorization.rs   # Auth utilities and helpers
├── author.rs          # DMO + AuthorTableTrait
├── cache.rs           # Cache abstractions and traits
├── health.rs          # Health check traits
├── krate.rs           # DMO + CrateTableTrait
├── mail.rs            # DMO + MailerTrait
├── password.rs        # DMO + PasswordHasherTrait
├── role_code.rs       # DMO + RoleCodeTableTrait
└── mod.rs             # Public-facing API: re-exports all DMO + traits
```

### Domain Module Notes

- `mod.rs` serves as the central API for the domain layer, re-exporting only the public-facing traits and types (DMOs).
- Internal scoping (e.g., `pub(super)`) is used where possible to restrict visibility, while maintaining module usability.
- `authorization.rs` provides auth utilities and credential handling
- `cache.rs` and `health.rs` provide infrastructure abstractions for caching and diagnostics

### `src/repository/`

- SQLx-backed implementations of the traits, the types are not to be used outside of src/repository/

```
src/repository/
├── app_user_sqlx.rs   # SQLx-backed user authentication
├── author_sqlx.rs     # SQLx-backed author management
├── crate_sqlx.rs      # SQLx-backed crate management  
├── database.rs        # Database connection and lifecycle
├── env.rs             # Environment configuration
├── health_check.rs    # System diagnostics implementation
├── redis_cache.rs     # Redis caching implementation
├── role_code_mapping.rs # Static role definitions
├── role_code_sqlx.rs  # SQLx-backed role management
└── mod.rs             # Public interface layer for repository consumers
```

---

## Design Principles

| Layer        | Aware Of             | Unaware Of          |
|--------------|----------------------|---------------------|
| `domain/`    | Traits               | SQLx, mocks         |
| `repository/`| SQLx                 | Mocks               |
| `mock/`      | Mocks                | SQLx                |

The design applies the **Dependency Inversion Principle**, ensuring all high-level logic depends on interfaces, not implementations.

---

## Schema Initialization

The `cr8s` schema and default roles (`Admin`, `Editor`, `Viewer`) are loaded using the CLI:

```bash
# Docker compose (recommended - handles environment automatically)
docker compose run --rm cli load-schema

# Local cargo (requires manual environment and path setup)
export DATABASE_URL="postgres://postgres:secret@localhost:5432/cr8s"
export REDIS_URL="redis://127.0.0.1:6379/"
export CR8S_DB_INIT_SQL="scripts/sql/db-init.sql"
cargo run --bin cli -- load-schema
```

This executes `scripts/sql/db-init.sql` in full, and ensures the `role` table is pre-populated.

Use `CR8S_DB_INIT_SQL=/path/to/alt.sql` to override the default file.
