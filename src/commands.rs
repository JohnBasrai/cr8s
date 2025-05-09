use anyhow::{anyhow, Context, Result};
use std::str::FromStr;

use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use tera::Tera;

use crate::auth;
use crate::mail::HtmlMailer;
use crate::models::{NewUser, RoleCode};
use crate::repositories::{CrateRepository, RoleRepository, UserRepository};

async fn load_db_connection() -> Result<AsyncPgConnection> {
    // ---
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => {
            eprintln!("Cannot connect to Postgres:{err}");
            return Err(anyhow!(err));
        }
    };

    match AsyncPgConnection::establish(&database_url).await {
        Ok(conn) => Ok(conn),
        Err(err) => {
            let msg = format!("Cannot connect to Postgres:{err}");
            eprintln!("{}", msg);
            Err(anyhow!(msg))
        }
    }
}

pub async fn create_user(
    username: String,
    password: String,
    role_codes: Vec<String>,
) -> Result<()> {
    // ---
    let mut c = load_db_connection().await?;

    let password_hash = auth::hash_password(password)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))?;

    let new_user = NewUser {
        username,
        password: password_hash,
    };

    let role_enums: Vec<RoleCode> = role_codes
        .iter()
        .map(|v| RoleCode::from_str(v.as_str()).map_err(|_| anyhow!("Invalid role code: {}", v)))
        .collect::<Result<_, _>>()?;

    let user = UserRepository::create(&mut c, new_user, role_enums).await?;

    println!("User created {:?}", user);
    let roles = RoleRepository::find_by_user(&mut c, &user).await?;
    println!("Roles assigned {:?}", roles);
    Ok(())
}

/// Fetches all users along with their assigned roles and returns a
/// formatted table as a vector of strings.
///
/// Each line includes the username, ID, creation timestamp, and
/// a comma-separated list of role codes. The output is width-adjusted
/// to fit the longest username for clean column alignment.
///
/// This function does not print anything; the caller is responsible
/// for displaying or redirecting the formatted output.
pub async fn list_users_formatted() -> Result<Vec<String>> {
    // ---
    let mut c = load_db_connection().await?;
    let users_with_roles = UserRepository::find_with_roles(&mut c)
        .await
        .with_context(|| "Failed to list users")?;

    let max_username_len = users_with_roles
        .iter()
        .map(|(user, _)| user.username.len())
        .max()
        .unwrap_or(10);

    let username_col_width = max_username_len + 2;
    let mut lines = Vec::with_capacity(2 + users_with_roles.len());

    // Header
    let header = format!(
        "{:<width$} {:<6} {:<20} {}",
        "Username",
        "ID",
        "Created At",
        "Roles",
        width = username_col_width
    );
    lines.push(header);

    let total_width = username_col_width + 6 + 20 + 2 + 12;
    lines.push("-".repeat(total_width));

    for (user, roles) in users_with_roles {
        let role_labels = roles
            .iter()
            .map(|(_, role)| role.code.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        lines.push(format!(
            "{:<width$} {:<6} {:<20} {}",
            user.username,
            user.id,
            user.created_at.format("%Y-%m-%d %H:%M"),
            role_labels,
            width = username_col_width
        ));
    }

    Ok(lines)
}

pub async fn user_exists(user: &str) -> Result<bool> {
    // ---
    let mut c = load_db_connection().await?;
    let user = user.to_owned();

    // TODO: find_by_username should take &str, someday
    let _ = UserRepository::find_by_username(&mut c, &user).await?;

    Ok(true)
}

/// Deletes a user from the database by numeric ID.
///
/// Used internally by the CLI to support `users delete <id>` syntax.
/// Succeeds silently if no user with the given ID exists.
pub async fn delete_user_by_id(user_id: i32) -> Result<()> {
    // ---
    use crate::schema::users::dsl::*;
    let mut conn = load_db_connection().await?;
    diesel::delete(users.filter(id.eq(user_id)))
        .execute(&mut conn)
        .await?;
    Ok(())
}

/// Deletes a user from the database by username.
///
/// Enables CLI usage like `users delete <username>`.
/// Succeeds silently if the username does not exist.
pub async fn delete_user_by_username(name: &str) -> Result<()> {
    // ---
    use crate::schema::users::dsl::*;
    let mut conn = load_db_connection().await?;
    diesel::delete(users.filter(username.eq(name)))
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub async fn digest_send(email: String, hours_since: i32) -> Result<()> {
    // ---
    let mut c = load_db_connection().await?;
    let tera = Tera::new("templates/**/*.html").context("Cannot load template engine")?;
    let crates = CrateRepository::find_since(&mut c, hours_since).await?;

    if !crates.is_empty() {
        println!("Sending digest for {} crates", crates.len());

        let hostname = std::env::var("SMTP_HOST").context("Missing SMTP_HOST env var")?;
        let username = std::env::var("SMTP_USERNAME").context("Missing SMTP_USERNAME env var")?;
        let password = std::env::var("SMTP_PASSWORD").context("Missing SMTP_PASSWORD env var")?;

        let mailer = HtmlMailer::new(tera, hostname, username, password)?;

        mailer
            .send_digest(&email, &crates)
            .map_err(|e| anyhow::anyhow!("Failed to send email: {:?}", e))?;
    }
    Ok(())
}
