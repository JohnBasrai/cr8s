use crate::domain::{Role, RoleCode, RoleCodeTableTrait, RoleCodeTableTraitPtr};
use crate::repository::{get_pool, RoleCodeMapping};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::instrument;

// ---

#[derive(Debug)]
pub struct RoleCodeRepo {
    pool: PgPool,
}

// ---

impl RoleCodeRepo {
    pub fn new(pool: PgPool) -> Self {
        // ---
        Self { pool }
    }
}

// ---

pub fn create_role_code_repo() -> Result<RoleCodeTableTraitPtr> {
    // ---
    Ok(Arc::new(RoleCodeRepo::new(get_pool().clone())))
}

// ---

#[async_trait]
impl RoleCodeTableTrait for RoleCodeRepo {
    // ---
    #[instrument(skip(self))]
    async fn find_all(&self) -> Result<Vec<RoleCode>> {
        // ---
        // TODO I think this method is not used. -jbasrai
        // ---
        // ORDER BY id for stable ordering; id is not included in SELECT

        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT code AS "code: RoleCodeMapping"
            FROM role
            ORDER BY id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| RoleCode::from(r.code)).collect())
    }

    // ---

    #[instrument(skip(self))]
    async fn find_role_name_by_code(&self, code: RoleCode) -> Result<Role> {
        // ---
        let row: RoleRow = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, code as "code: RoleCodeMapping", name
            FROM role
            WHERE code = $1::"RoleCodeMapping"
            "#,
        )
        .bind(RoleCodeMapping::from(code))
        .fetch_one(&self.pool)
        .await?;

        Ok(Role {
            id: row.id,
            code: row.code.into(),
            name: row.name,
        })
    }

    // ---

    #[instrument(skip(self))]
    async fn find_role_codes_by_user(&self, user_id: i32) -> Result<Vec<RoleCode>> {
        // ---

        let rows: Vec<RoleRow> = sqlx::query_as(
            r#"
            SELECT r.code AS "code: RoleCodeMapping"
            FROM role r
            JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            ORDER BY r.id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| RoleCode::from(r.code)).collect())
    }
}

#[derive(Debug, serde::Deserialize, sqlx::FromRow)]
struct RoleRow {
    id: i32,
    code: RoleCodeMapping,
    name: String,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Role {
            id: row.id,
            code: row.code.into(),
            name: row.name,
        }
    }
}
