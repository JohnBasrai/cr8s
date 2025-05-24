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
- Easy swapping of implementations (e.g., Diesel vs. in-memory)
- Fully mockable testing
- Clear separation between domain logic and infrastructure

### 3. Clean Architecture / Hexagonal Architecture
Each layer has one clear responsibility:
- `domain/`: core interfaces and shared data types
- `repository/`: Diesel-backed implementations
- `mock/`: test-only, in-memory mocks
- `service/`: business logic orchestration
- `auth/`, `mail/`: side-effecting adapters and helpers

### 4. SOLID Principles in Practice
- **S**ingle Responsibility: Each module has a focused purpose
- **O**pen/Closed: Traits can be extended via new impls
- **L**iskov Substitution: Swappable via trait objects
- **I**nterface Segregation: Narrow traits instead of catch-all services
- **D**ependency Inversion: Central organizing idea

### 5. Separation of Concerns
- CLI, HTTP, database, and auth layers are decoupled
- Modules expose traits and inject dependencies — not global state

### 6. Repository Layer

Core Diesel-backed types (e.g. `AppUser`, `UserRole`) are defined in `repository/diesel.rs` and selectively re-exported through `repository/mod.rs` for use in CLI, service, or test layers.

Trait implementations are housed in `repository/diesel.rs`, while orchestration and helper logic lives in `repository/provider.rs`. A central `mod.rs` re-exports the public API.

A new domain-facing abstraction, `DBContextTrait`, was introduced to encapsulate all database lifecycle logic.

This trait has no public methods yet — it simply acts as an opaque handle to a lazily-initialized global database context. It is returned via `initialize_database()` and accessed via `get_database()`.

All state (connection pool, etc.) is hidden behind `Box<dyn DBContextTrait>`, and the implementation lives entirely in `repository/database.rs`. This ensures that both `cli.rs` and `server.rs` can initialize and access the database without being coupled to Diesel or Rocket internals.

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
├── diesel.rs          # Diesel-backed data models and trait impls
├── provider.rs        # Higher-level repository orchestration
└── mod.rs             # Central API exposing public repository symbols
```

This modular structure allows the repository layer to grow while preserving a stable public interface through `mod.rs`.
The `ServerInfoTrait` provides a clean abstraction for querying diagnostics like server IP. Its Diesel-based implementation uses raw SQL for operations not covered by Diesel’s query builder.

This layering ensures:
- Diesel remains encapsulated and swappable
- Domain code never depends directly on schema or ORM details
- Repository logic is testable with in-memory or mock trait impls

---

## 🧠 Additional Architectural Models Applied

| Architecture Style           | Applies? | Notes |
|-----------------------------|----------|-------|
| Enterprise Software          | ✅ Yes  | Roles, users, emails, external systems |
| Domain-Driven Design (lite)  | ✅ Yes  | Domain traits express business capabilities |
| Service-Oriented Design      | ✅ Yes  | Coordinated modules like `user.rs`, `digest.rs` |
| Onion/Clean/Layered Arch     | ✅ Yes  | Core-to-edges dependency flow |
| CQRS / Event Sourcing        | ⚠️ Partial | Could be layered on later |
| Actor-based async systems    | ⚠️ Not yet | Future candidate for service messaging |

---

## 🧾 Future Considerations

- Add GraphQL or REST API adapters over domain services
- Introduce event emitters / subscribers for decoupled coordination
- Move to multi-tenant / namespaced schema if needed

---

CR8S is designed to grow — with structure, confidence, and testability.
