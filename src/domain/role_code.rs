// domain/role_code.rs

//! Contains a trait for querying roles and mapping them to users.
//! Contains a domain-level enum for user role classification.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Role {
    pub id: i32,
    pub code: RoleCode,
    pub name: String,
}

/// Distinct access roles available to application users.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum RoleCode {
    Admin,
    Editor,
    Viewer,
}

impl fmt::Display for RoleCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RoleCode::Admin => "Admin",
            RoleCode::Editor => "Editor",
            RoleCode::Viewer => "Viewer",
        };
        write!(f, "{s}")
    }
}

impl FromStr for RoleCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(RoleCode::Admin),
            "Editor" => Ok(RoleCode::Editor),
            "Viewer" => Ok(RoleCode::Viewer),
            _ => Err(()),
        }
    }
}

#[async_trait::async_trait]
pub trait RoleCodeTableTrait: Send + Sync {
    // ---
    async fn find_role_codes_by_user(&self, user_id: i32) -> Result<Vec<RoleCode>>;
    async fn find_all(&self) -> Result<Vec<RoleCode>>;
    async fn find_role_name_by_code(&self, code: RoleCode) -> Result<Role>;
}

pub type RoleCodeTableTraitPtr = Arc<dyn RoleCodeTableTrait + Send + Sync>;

pub use crate::repository::create_role_code_repo;
