// cr8s/src/repository/crate.rs

use crate::domain::{Crate, CrateSummary, CrateTableTrait, CrateTableTraitPtr, NewCrate};
use crate::repository::get_pool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;

// ---

#[derive(Debug, Clone)]
pub struct CrateRepo {
    pool: sqlx::PgPool,
}

// ---

impl CrateRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        // ---
        Self { pool }
    }
}

// ---

pub fn create_crate_repo() -> CrateTableTraitPtr {
    // --
    let pool = get_pool().clone();
    Arc::new(CrateRepo::new(pool))
}

// ---

#[async_trait]
impl CrateTableTrait for CrateRepo {
    // ---

    async fn find_multiple(&self, limit: i64) -> Result<Vec<Crate>> {
        // ---
        let rows = sqlx::query_as::<_, CrateRow>(
            r#"
        SELECT id, author_id, code, name, version, description, created_at
        FROM crate
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("CrateTableTrait::find_multiple")?;

        let crates: Vec<Crate> = rows.into_iter().map(Into::into).collect();
        Ok(crates)
    }

    // ---
    async fn find(&self, id: i32) -> Result<Crate> {
        // ---
        let row = sqlx::query_as::<_, CrateRow>(
            r#"
        SELECT id, author_id, code, name, version, description, created_at
        FROM crate
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("CrateTableTrait::find")?;

        Ok(row.into())
    }

    // ---

    async fn create(&self, new_crate: NewCrate) -> Result<Crate> {
        // ---
        let rec = sqlx::query_as::<_, CrateRow>(
            r#"
            INSERT INTO crate (author_id, code, name, version, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, author_id, code, name, version, description, created_at
            "#,
        )
        .bind(new_crate.author_id)
        .bind(new_crate.code)
        .bind(new_crate.name)
        .bind(new_crate.version)
        .bind(new_crate.description)
        .fetch_one(&self.pool)
        .await
        .context("CrateTableTrait::create")?;

        Ok(rec.into())
    }

    // ---

    /// Update an existing crate by ID.
    async fn update(&self, id: i32, updated: NewCrate) -> Result<Crate> {
        // ---
        let rec = sqlx::query_as::<_, CrateRow>(
            r#"
        UPDATE crate
        SET author_id   = $1,
            code        = $2,
            name        = $3,
            version     = $4,
            description = $5
        WHERE id = $6
        RETURNING id, author_id, code, name, version, description, created_at
        "#,
        )
        .bind(updated.author_id)
        .bind(updated.code)
        .bind(updated.name)
        .bind(updated.version)
        .bind(updated.description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("CrateTableTrait::update")?;

        Ok(rec.into())
    }

    // ---

    async fn delete(&self, id: i32) -> Result<()> {
        // ---
        sqlx::query(
            r#"
            DELETE FROM crate
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("CrateTableTrait::delete")?;

        Ok(())
    }

    // ---

    /// Return crate summaries modified within the last N hours.
    async fn find_since(&self, hours_since: i32) -> Result<Vec<CrateSummary>> {
        // ---
        let records = sqlx::query_as::<_, CrateSummaryRow>(
            r#"
            SELECT name, version
            FROM crate
            WHERE created_at >= NOW() - ($1 * INTERVAL '1 hour')::INTERVAL
            "#,
        )
        .bind(hours_since as f64)
        .fetch_all(&self.pool)
        .await
        .context("CrateTableTrait::find_since")?;

        Ok(records.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CrateRow {
    id: i32,
    author_id: i32,
    code: String,
    name: String,
    version: String,
    description: Option<String>,
    created_at: chrono::NaiveDateTime,
}

impl From<CrateRow> for Crate {
    fn from(row: CrateRow) -> Self {
        Crate {
            id: row.id,
            author_id: row.author_id,
            code: row.code,
            name: row.name,
            version: row.version,
            description: row.description,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CrateSummaryRow {
    name: String,
    version: String,
}

impl From<CrateSummaryRow> for CrateSummary {
    fn from(row: CrateSummaryRow) -> Self {
        CrateSummary {
            name: row.name,
            version: row.version,
        }
    }
}
