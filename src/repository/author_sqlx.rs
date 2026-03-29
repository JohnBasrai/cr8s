use crate::domain::{Author, AuthorTableTrait, AuthorTableTraitPtr, NewAuthor};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use std::sync::Arc;

// ---
pub fn create_author_repo() -> AuthorTableTraitPtr {
    // --
    let pool = super::get_pool();
    Arc::new(AuthorRepo { pool: pool.clone() })
}

#[derive(Debug, Clone)]
pub struct AuthorRepo {
    pool: PgPool,
}

// ---
#[derive(Debug, FromRow)]
struct AuthorRow {
    id: i32,
    name: String,
    email: String,
    created_at: chrono::NaiveDateTime,
    row_version: i32,
}

impl From<AuthorRow> for Author {
    fn from(row: AuthorRow) -> Self {
        Author {
            id: row.id,
            name: row.name,
            email: row.email,
            created_at: row.created_at,
            row_version: row.row_version,
        }
    }
}

#[async_trait]
impl AuthorTableTrait for AuthorRepo {
    // ---
    async fn create(&self, author: NewAuthor) -> Result<Author> {
        // ---
        let row = sqlx::query_as::<_, AuthorRow>(
            r#"
            INSERT INTO author (name, email)
            VALUES ($1, $2)
            RETURNING id, name, email, created_at
            "#,
        )
        .bind(&author.name)
        .bind(&author.email)
        .fetch_one(&self.pool)
        .await
        .context("AuthorRepo::create failed")?;

        Ok(row.into())
    }

    // ---
    async fn update(&self, id: i32, current_version: i32, author: Author) -> Result<Author> {
        // ---
        let row = sqlx::query_as::<_, AuthorRow>(
            r#"
            UPDATE author
            SET name = $1, email = $2, row_version = row_version + 1
            WHERE id = $3 AND row_version = $4
            RETURNING id, name, email, created_at, row_version
            "#,
        )
        .bind(&author.name)
        .bind(&author.email)
        .bind(id)
        .bind(current_version)
        .fetch_one(&self.pool)
        .await
        .context("AuthorRepo::update failed")?;

        Ok(row.into())
    }

    // ---
    async fn find(&self, id: i32) -> Result<Author> {
        // ---
        let author = sqlx::query_as::<_, AuthorRow>(
            r#"
            SELECT id, name, email, created_at
            FROM author
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("AuthorRepo::find: no author found with id {id}"))?;

        Ok(author.into())
    }

    // ---
    async fn find_multiple(&self, limit: i64) -> Result<Vec<Author>> {
        // ---
        let authors = sqlx::query_as::<_, AuthorRow>(
            r#"
            SELECT id, name, email, created_at
            FROM author
            ORDER BY id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("AuthorRepo::find_multiple query failed")?;

        Ok(authors.into_iter().map(Into::into).collect())
    }

    // ---
    async fn delete(&self, id: i32) -> Result<()> {
        // ---
        sqlx::query(
            r#"
            DELETE FROM author
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("AuthorRepo::delete: failed for id {id}"))?;

        Ok(())
    }
}
