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

### Joinable Macros
```rust
diesel::joinable!(app_user -> author (author_id));
diesel::joinable!(crate_ -> author (author_id));
diesel::joinable!(user_role -> app_user (user_id));
diesel::joinable!(user_role -> role (role_id));
```

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
- No awareness of Diesel or mocks

```
src/domain/
├── app_user.rs        // DMO + AppUserTableTrait
├── author.rs          // DMO + AuthorTableTrait
├── krate.rs           // DMO + CrateTableTrait
├── mail.rs            // DMO + MailerTrait
├── password.rs        // DMO + PasswordHasherTrait
├── role_code.rs       // DMO + RoleCodeTableTrait
├── user_role.rs       // DMO + UserRoleTableTrait
├── server_info.rs     // Diagnostic trait: ServerInfoTrait
└── mod.rs             // Public-facing API: re-exports all DMO + traits
```

### Domain Module Notes

- `mod.rs` serves as the central API for the domain layer, re-exporting only the public-facing traits and types (DMOs).
- Internal scoping (e.g., `pub(super)`) is used where possible to restrict visibility, while maintaining module usability.
- `ServerInfoTrait` is not tied to a specific table but provides metadata access (e.g., server IP). Its role is similar to a system utility service, useful for diagnostics or CLI tooling.

### `src/repository/`

- Diesel-backed implementations of the traits, the types are not to be used out side of src/repository/

```
src/repository/
├── diesel.rs          // Diesel-backed types and trait impls
├── provider.rs        // Higher-level repository orchestration
└── mod.rs             // Public interface layer for repository consumers
```

### `src/mock/` (optional)
- In-memory mock implementations for testing

```
src/mock/
├── mock_app_user.rs
├── mock_author.rs
...
```

---

## Design Principles

| Layer        | Aware Of             | Unaware Of          |
|--------------|----------------------|---------------------|
| `domain/`    | Traits               | Diesel, mocks       |
| `repository/`| Diesel               | Mocks               |
| `mock/`      | Mocks                | Diesel              |

The design applies the **Dependency Inversion Principle**, ensuring all high-level logic depends on interfaces, not implementations.

---

## Schema Initialization

The `cr8s` schema and default roles (`Admin`, `Editor`, `Viewer`) are loaded using the CLI:

```bash
# All of these will work
cargo run --bin cli -- load-schema
docker compose run cli load-schema

This executes `scripts/sql/db-init.sql` in full, and ensures the `role` table is pre-populated.

Use `CR8S_DB_INIT_SQL=/path/to/alt.sql` to override the default file.

