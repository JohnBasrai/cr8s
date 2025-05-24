// src/domain/app_user.rs
//! Domain-facing trait and types for application users.
//!
//! This module defines a clean interface and data model for user
//! management and role assignment logic. It is free of database,
//! serialization, or framework-specific concerns.

use super::RoleCode;
use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppUser {
    pub id: i32,
    pub username: String,
    pub(crate) password: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthenticatedAppUser {
    pub user: AppUser,
    pub role_codes: Vec<RoleCode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

pub type AppUserWithRoleCodes = (AppUser, Vec<RoleCode>);

#[async_trait::async_trait]
pub trait AppUserTableTrait: Send + Sync {
    // ---

    /// Create a new user with the given roles, roles may be empty.
    async fn create(&self, new_user: NewUser, role_codes: Vec<RoleCode>) -> Result<AppUser>;

    /// Find by unique user id
    async fn find(&self, id: i32) -> Result<AppUser>;

    /// Find the roles that this user has.
    async fn find_roles_by_user(&self, user: &AppUser) -> Result<Vec<RoleCode>>;

    /// Deletes a user from the system by their unique ID.
    async fn delete_by_id(&self, user_id: i32) -> Result<()>;

    /// Deletes a user by their unique username.
    async fn delete_by_username(&self, username: &str) -> Result<()>;

    /// Finds a user by their unique username.
    async fn find_by_username(&self, username: &str) -> Result<AppUser>;

    /// Finds all users and their roles.
    async fn find_with_roles(&self) -> Result<Vec<AppUserWithRoleCodes>>;
}

/// Shared trait object for user data access.
///
/// This domain module intentionally does not define a constructor for
/// `AppUserTableTraitPtr`, because construction requires Rocket’s `DbConn`, which is an
/// infrastructure concern.  Since the `/login` API in `rocket_routes` already has access
/// to `DbConn`, it can call `repository::create_app_user_repo(DbConn)` directly to create
/// an instance of this pointer type.
pub type AppUserTableTraitPtr = Arc<dyn AppUserTableTrait + Send + Sync>;

pub use crate::repository::create_app_user_repo;
