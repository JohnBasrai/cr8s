// src/repository/mod.rs
//! SQLx-backed repository implementations for all database and cache access.
//!
//! This module forms the infrastructure layer beneath the domain traits.
//!
//! ## Public interface (used by CLI, server, and tests)
//! - `create_app_user_repo`
//! - `create_author_repo`
//! - `create_crate_repo`
//! - `create_role_code_repo`
//! - `create_cache_health_service`
//! - `create_database_health_service`
//! - `init_cache_with_retry_from_env`
//! - `init_database_with_retry_from_env`
//! - `load_schema_from_sql_file`
//!
//! ## Internal-only symbols (not re-exported)
//! - `get_pool`, `get_redis_pool`
//! - individual `*Repo` structs
//! - `RoleCodeMapping` lookup logic

mod app_user_sqlx;
mod author_sqlx;
mod crate_sqlx;
mod database;
#[macro_use]
mod env;
mod health_check;
mod redis_cache;
mod role_code_mapping;
mod role_code_sqlx;

// --- Public interface
pub use app_user_sqlx::create_app_user_repo;
pub use author_sqlx::create_author_repo;
pub use crate_sqlx::create_crate_repo;
pub use database::init_database_with_retry_from_env;
pub use database::load_schema_from_sql_file;
pub use health_check::create_cache_health_service;
pub use health_check::create_database_health_service;
pub use redis_cache::create_cache_context;
pub use redis_cache::init_redis_cache_with_retry_from_env as init_cache_with_retry_from_env;
pub use role_code_sqlx::create_role_code_repo;

// --- Package scope interface
use database::get_pool;
use redis_cache::get_redis_pool;
use role_code_mapping::RoleCodeMapping;
