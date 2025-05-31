use crate::domain::{
    AppUser, AppUserTableTrait, AppUserTableTraitPtr, AppUserWithRoleCodes, NewUser, RoleCode,
};
use crate::repository::RoleCodeMapping;
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_app_user_repo() -> AppUserTableTraitPtr {
    // --
    let pool = super::get_pool();
    Arc::new(AppUserRepo { pool: pool.clone() })
}

#[derive(Debug, Clone)]
pub struct AppUserRepo {
    pool: PgPool,
}

// ---
#[derive(Debug, FromRow)]
struct AppUserRow {
    id: i32,
    username: String,
    password: String,
    created_at: chrono::NaiveDateTime,
}

impl From<AppUserRow> for AppUser {
    fn from(row: AppUserRow) -> Self {
        AppUser {
            id: row.id,
            username: row.username,
            password: row.password,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct RoleCodeRow {
    code: RoleCodeMapping,
}

#[async_trait]
impl AppUserTableTrait for AppUserRepo {
    // ---
    async fn create(&self, new_user: NewUser, role_codes: Vec<RoleCode>) -> Result<AppUser> {
        // ---
        let mut tx = self.pool.begin().await?;

        let rec = sqlx::query_as::<_, AppUserRow>(
            r#"
            INSERT INTO app_user (username, password)
            VALUES ($1, $2)
            RETURNING id, username, password, created_at
            "#,
        )
        .bind(&new_user.username)
        .bind(&new_user.password)
        .fetch_one(&mut *tx)
        .await?;

        for code in &role_codes {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                SELECT $1, id FROM role WHERE code = $2::"RoleCodeMapping"
                "#,
            )
            .bind(rec.id)
            .bind(code.to_string())
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to assign role: {code:?}"))?;
        }

        tx.commit().await?;
        Ok(rec.into())
    }

    // ---
    async fn find(&self, id: i32) -> Result<AppUser> {
        // ---
        let rec = sqlx::query_as::<_, AppUserRow>(
            r#"
            SELECT id, username, password, created_at
            FROM app_user
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("AppUserRepo::find: no user found with id {id}"))?;

        Ok(rec.into())
    }

    // ---
    async fn find_roles_by_user(&self, user: &AppUser) -> Result<Vec<RoleCode>> {
        // ---
        let rows = sqlx::query_as::<_, RoleCodeRow>(
            r#"
            SELECT r.code as "code: RoleCodeMapping"
            FROM user_roles ur
            JOIN role r ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user.id)
        .fetch_all(&self.pool)
        .await
        .with_context(|| {
            format!("AppUserRepo::find_roles_by_user: error getting roles for: {user:?}")
        })?;

        Ok(rows.into_iter().map(|row| row.code.into()).collect())
    }

    // ---
    async fn delete_by_id(&self, user_id: i32) -> Result<()> {
        // ---
        sqlx::query(r#"DELETE FROM app_user WHERE id = $1"#)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("AppUserRepo::delete_by_id error for: {user_id}"))?;

        Ok(())
    }

    // ---
    async fn delete_by_username(&self, username: &str) -> Result<()> {
        // ---
        sqlx::query(r#"DELETE FROM app_user WHERE username = $1"#)
            .bind(username)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!("AppUserRepo::delete_by_username failed for username: {username}")
            })?;

        Ok(())
    }

    // ---
    async fn find_by_username(&self, username: &str) -> Result<AppUser> {
        // ---
        let user = sqlx::query_as::<_, AppUserRow>(
            r#"
            SELECT id, username, password, created_at
            FROM app_user
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .with_context(|| {
            format!("AppUserRepo::find_by_username: no user found with username: {username}")
        })?;

        Ok(user.into())
    }

    // ---
    async fn find_with_roles(&self) -> Result<Vec<AppUserWithRoleCodes>> {
        // ---
        #[derive(FromRow)]
        struct AppUserWithRoleRow {
            user_id: i32,
            username: String,
            password: String,
            created_at: chrono::NaiveDateTime,
            code: Option<RoleCodeMapping>,
        }

        let rows = sqlx::query_as::<_, AppUserWithRoleRow>(
            r#"
            SELECT
              u.id as user_id,
              u.username,
              u.password,
              u.created_at,
              r.code as code
            FROM app_user u
            LEFT JOIN user_roles ur ON u.id = ur.user_id
            LEFT JOIN role r ON r.id = ur.role_id
            ORDER BY u.id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("AppUserRepo::find_with_roles: query or grouping failure")?;

        let mut grouped: HashMap<i32, (AppUser, Vec<RoleCode>)> =
            HashMap::with_capacity(rows.len());

        for row in rows {
            let entry = grouped.entry(row.user_id).or_insert_with(|| {
                (
                    AppUser {
                        id: row.user_id,
                        username: row.username.clone(),
                        password: row.password.clone(),
                        created_at: row.created_at,
                    },
                    Vec::new(),
                )
            });

            if let Some(code) = row.code {
                entry.1.push(code.into());
            }
        }

        Ok(grouped.into_values().collect())
    }
}
