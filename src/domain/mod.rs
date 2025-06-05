// src/domain/mod.rs
//! Domain trait interfaces and data types used across the application.
//!
//! The `domain` module defines the abstract behaviors and shared types for all core
//! concepts in the system, such as:
//!
//! | Area          | Module          | Description                                        |
//! | ------------- | --------------  | -------------------------------------------------- |
//! | authorization | `authorization` | Login identity and authentication behavior         |
//! | Contributors  | `author`        | Rust ecosystem contributors and authorship logic   |
//! | Crates        | `crate_`        | Published Rust crates and associated metadata      |
//! | RBAC          | `role`, `user_role` | Role-based access control and assignment       |
//! | Passwords     | `password`      | Password hashing and credential validation         |
//! | Email         | `mail`          | Outbound email dispatching via `MailerTrait`       |
//!
//! Traits in this layer define the **expected behavior** without prescribing the
//! implementation, enabling the repository and mock layers to provide concrete
//! versions. This enables testing, service composition, and runtime flexibility while
//! keeping high-level logic decoupled from infrastructure.
//!
//! ✅ Do this (preferred):
//!
//! ```rust
//! use crate::domain::{RoleCode, RoleCodeTableTrait};
//! ```
//!
//! ❌ Not this (bypasses domain's public API):
//!
//! ```rust
//! use crate::domain::role_code::{RoleCode, RoleCodeTableTrait};
//! ```

// Re-export domain models and traits for external use

mod app_user;
mod author;
mod authorization;
mod cache;
mod health;
mod krate;
mod mail;
mod password;
mod role_code;

pub use app_user::{
    //
    create_app_user_repo,
    AppUser,
    AppUserTableTrait,
    AppUserTableTraitPtr,
    AppUserWithRoleCodes,
    NewUser,
};

pub use authorization::{
    //
    authenticate_user,
    create_password_hasher,
    Credentials,
    LoginError,
};

pub use author::{
    //
    create_author_repo,
    Author,
    AuthorTableTrait,
    AuthorTableTraitPtr,
    NewAuthor,
};

pub use cache::{create_cache_context, CacheContextTrait, CacheContextTraitPtr};

pub use health::{
    // ---
    create_cache_health_service,
    create_database_health_service,
    HealthTrait,
    HealthTraitPtr,
};
pub use krate::{
    //
    create_crate_repo,
    Crate,
    CrateSummary,
    CrateTableTrait,
    CrateTableTraitPtr,
    NewCrate,
};
pub use mail::{create_mailer, MailerTrait, MailerTraitPtr};
pub use password::{PasswordHasherTrait, PasswordHasherTraitPtr};
pub use role_code::{
    //
    create_role_code_repo,
    Role,
    RoleCode,
    RoleCodeTableTrait,
    RoleCodeTableTraitPtr,
};

/// Public hook exposed to CLI/server to initialize DB at startup.
pub async fn init_database_with_retry_from_env() -> anyhow::Result<()> {
    crate::repository::init_database_with_retry_from_env().await
}

/// Public hook exposed to CLI/server to initialize cache at startup.
pub async fn init_cache_with_retry_from_env() -> anyhow::Result<()> {
    crate::repository::init_cache_with_retry_from_env().await
}

pub use crate::repository::load_schema_from_sql_file;
