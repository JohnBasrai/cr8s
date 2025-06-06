# CR8S – Architecture Overview

This document outlines the high-level software architecture principles used in the CR8S backend system.

## ✅ Key Design Patterns & Principles

### 1. Trait-Based Object-Oriented Design
Rust traits are used to define clear, testable interfaces for all key abstractions:
- `AppUserTableTrait`
- `PasswordHasherTrait`
- `MailerTrait`
- `ServerInfoTrait`
- ...

These are equivalent to abstract base classes in OOP languages like C++ or Java.

### 2. Dependency Inversion Principle (DIP)
High-level modules (services, commands) depend on traits defined in `src/domain/`, not on concrete implementations. This allows:
- Easy swapping of implementations (e.g., SQLx to in-memory)
- Fully mockable testing
- Clear separation between domain logic and infrastructure

### 3. Clean Architecture / Hexagonal Architecture

Each layer has one clear responsibility:

| Module           | Responsibility                         |
|----------------- | -------------------------------------- |
| `domain/`        | core interfaces and shared data types  |
| `repository/`    | SQLx-backed implementations            |
| `rocket_routes/` | HTTP endpoint handlers                  |
| `auth.rs`, `mail/` | side-effecting adapters and helpers  |

### 4. SOLID Principles in Practice
- **S**ingle Responsibility: Each module has a focused purpose
- **O**pen/Closed: Traits can be extended via new impls
- **L**iskov Substitution: Swappable via trait objects
- **I**nterface Segregation: Narrow traits instead of catch-all services
- **D**ependency Inversion: Central organizing idea

### 5. Separation of Concerns

The repository layer demonstrates clear separation of concerns through focused, single-responsibility modules:

| File                     | Purpose                                    |
|-------------------------|-------------------------------------------- |
| `app_user_sqlx.rs`      | User authentication and account management  |
| `author_sqlx.rs`        | Rust community author/contributor data      |
| `crate_sqlx.rs`         | Rust crate/package metadata                 |
| `role_code_sqlx.rs`     | Permission and role-based access control    |
| `database.rs`           | Database connection and lifecycle management |
| `load_schema_from_sql_file()` | Loads schema and default roles from SQL file   |
| `redis_cache.rs`        | Session and ephemeral data caching          |
| `env.rs`                | Environment configuration management        |
| `health_check.rs`       | System diagnostics and monitoring           |
| `role_code_mapping.rs`  | Static role definitions and mappings        |
| `mod.rs`                | Public API and trait re-exports             |

This modular design ensures that:
- CLI, HTTP, database, and auth layers are decoupled
- Each concern is isolated to its own module
- Modules expose traits and inject dependencies — not global state
- Infrastructure details remain hidden behind domain interfaces

### 6. Repository Layer

The `repository/database.rs` module also includes `load_schema_from_sql_file()`, 
which is invoked by the CLI `load-schema` command. It reads a SQL file (defaulting 
to `scripts/sql/db-init.sql`), initializes the connection pool if necessary, and 
executes the script as a batch — including insertion of default roles. The path 
can be overridden using the `CR8S_DB_INIT_SQL` environment variable.


Core SQLx-backed types (e.g. `AppUser`, `UserRole`, etc.) are defined in `src/repository/*_sqlx.rs` files, each focused on a specific domain entity.

A new domain-facing abstraction, `DBContextTrait`, was introduced to encapsulate all database lifecycle logic.

This trait has no public methods yet — it simply acts as an opaque handle to a lazily-initialized global database context. It is returned via `initialize_database()` and accessed via `get_database()`.

All state (connection pool, etc.) is hidden behind `Box<dyn DBContextTrait>`, and the implementation lives entirely in `repository/database.rs`. This ensures that both `cli.rs` and `server.rs` can initialize and access the database without being coupled to SQLx or Rocket internals.

### 🧾 Route-to-Trait Dependency Matrix

| Route Function               | Required Trait Objects                          |
|-----------------------------|--------------------------------------------------|
| `login`                     | `AppUserTableTraitPtr`, `CacheContextTraitPtr`   |
| `health_endpoint`           | *(none)*                                         |
| `options`                   | *(none)*                                         |
| `me`                        | `AppUserTableTraitPtr`                           |
| `get_rustaceans`            | `AuthorTableTraitPtr`                            |
| `view_rustacean`            | `AuthorTableTraitPtr`                            |
| `create_rustacean`          | `AuthorTableTraitPtr`                            |
| `update_rustacean`          | `AuthorTableTraitPtr`                            |
| `delete_rustacean`          | `AuthorTableTraitPtr`                            |
| `get_crates`                | `CrateTableTraitPtr`                             |
| `view_crate`                | `CrateTableTraitPtr`                             |
| `create_crate`              | `CrateTableTraitPtr`, `AuthorTableTraitPtr`      |
| `update_crate`              | `CrateTableTraitPtr`                             |
| `delete_crate`              | `CrateTableTraitPtr`                             |


### Repository Module Layout (2025 Refactor)

```
src/repository/
├── app_user_sqlx.rs       # User authentication SQLx implementation
├── author_sqlx.rs         # Author/contributor SQLx implementation  
├── crate_sqlx.rs          # Crate metadata SQLx implementation
├── database.rs            # Database connection and lifecycle management
├── env.rs                 # Environment configuration
├── health_check.rs        # System diagnostics
├── redis_cache.rs         # Redis caching implementation
├── role_code_mapping.rs   # Static role definitions
├── role_code_sqlx.rs      # Role-based access control SQLx implementation
└── mod.rs                 # Central API exposing public repository symbols
```

This modular structure allows the repository layer to grow while preserving a stable public interface through `mod.rs`.
The `ServerInfoTrait` provides a clean abstraction for querying diagnostics like server IP. Its SQLx-based implementation uses raw SQL for operations not covered by SQLx's query builder.

This layering ensures:
- SQLx remains encapsulated and swappable
- Domain code never depends directly on database schemas or ORM details
- Repository logic is testable with in-memory or mock trait impls

### Testing Architecture

CR8S employs a comprehensive multi-layered testing strategy:

| Test Layer | Location | Purpose | Coverage |
|------------|----------|---------|----------|
| **Unit Tests** | `src/*/mod.rs` (inline) | Business logic, role validation | Guard role logic, trait implementations |
| **Integration Tests** | `tests/` | End-to-end workflows | CLI commands, HTTP API endpoints, authentication flows |
| **Domain Visibility** | `src/tests/` | API encapsulation | Trait visibility, module boundaries |

#### Integration Test Structure
```
tests/
├── cli_integration.rs     # CLI command testing via Docker Compose
├── server_integration.rs  # HTTP API endpoint testing
└── (unit tests in src/tests/ for architectural validation)
```

**Key Testing Principles:**
- **Unit tests** focus on isolated business logic with minimal mocks
- **Integration tests** validate complete workflows using real infrastructure (PostgreSQL, Redis, HTTP)
- **Complementary coverage** - unit tests skip complex integration scenarios, integration tests validate full stack
- **Playwright alignment** - server integration tests mirror frontend test requirements

This ensures both correctness of individual components and confidence in the complete system.

---

## 🧠 Additional Architectural Models Applied

| Architecture Style           | Applies? | Notes |
|-----------------------------|----------|-------|
| Enterprise Software          | ✅ Yes  | Roles, users, emails, external systems |
| Domain-Driven Design (lite)  | ✅ Yes  | Domain traits express business capabilities |
| Service-Oriented Design      | ✅ Yes  | Coordinated modules like `user.rs`, `digest.rs` |
| Onion/Clean/Layered Arch     | ✅ Yes  | Core-to-edges dependency flow |
| EMBP*                        | ✅ Yes  | **Boundary**: Gateway files (`mod.rs`) control public APIs, **Entity**: Domain types in focused modules, **Provider**: SQLx implementations behind trait boundaries |
| CQRS / Event Sourcing        | ⚠️ Partial | Could be layered on later |
| Actor-based async systems    | ⚠️ Not yet | Future candidate for service messaging |

> EMBP = Explicit Module Boundary Pattern

---

## 🧾 Future Considerations

- Add GraphQL or REST API adapters over domain services
- Introduce event emitters / subscribers for decoupled coordination
- Move to multi-tenant / namespaced schema if needed

---

CR8S is designed to grow — with structure, confidence, and testability.
