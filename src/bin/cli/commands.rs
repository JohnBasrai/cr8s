// src/bin/cli/commands.rs
// Command implementation logic - preserved from original commands.rs

use anyhow::{anyhow, Context, Result};
use cr8s::domain::{
    //
    create_app_user_repo,
    create_crate_repo,
    create_mailer,
    create_password_hasher,
    NewUser,
    RoleCode,
};

// ---

/// Creates a user with the specified roles.
pub async fn create_user(
    username: String,
    password: String,
    role_codes: Vec<RoleCode>,
) -> Result<()> {
    // ---

    let user_repo = create_app_user_repo();

    let password_hash = create_password_hasher()?
        .hash_password(&password)
        .map_err(|e| anyhow!("Password hashing failed: {e}"))?;

    let new_user = NewUser {
        username: username.clone(),
        password: password_hash,
    };

    // ---

    let user = user_repo
        .create(new_user, role_codes.clone())
        .await
        .with_context(|| format!("Failed to create user: {username}"))?;

    tracing::info!("User created: {:?}", user);
    println!("✅ Created user: {} (ID: {})", user.username, user.id);

    // ---

    if !role_codes.is_empty() {
        let role_names: Vec<String> = role_codes.iter().map(|r| format!("{:?}", r)).collect();
        println!("📝 Assigned roles: {}", role_names.join(", "));
    }

    Ok(())
}

// ---

/// Deletes a user from the database by numeric ID.  Succeeds silently
/// if no user with the given ID exists.
pub async fn delete_user_by_id(user_id: i32) -> Result<()> {
    // ---

    let user_repo = create_app_user_repo();
    user_repo
        .delete_by_id(user_id)
        .await
        .with_context(|| format!("Failed to delete user with ID: {}", user_id))?;

    println!("✅ Deleted user with ID: {}", user_id);
    Ok(())
}

// ---

/// Deletes a user from the database by username.
/// Succeeds silently if the username does not exist.
pub async fn delete_user_by_username(name: &str) -> Result<()> {
    // ---

    let user_repo = create_app_user_repo();
    user_repo
        .delete_by_username(name)
        .await
        .with_context(|| format!("Failed to delete user: {}", name))?;

    println!("✅ Deleted user: {}", name);
    Ok(())
}

// ---

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

    let user_repo = create_app_user_repo();
    let users_with_roles = user_repo
        .find_with_roles()
        .await
        .with_context(|| "Failed to list users")?;

    // ---

    let max_username_len = users_with_roles
        .iter()
        .map(|(user, _)| user.username.len())
        .max()
        .unwrap_or(10);

    let username_col_width = max_username_len + 2;
    let mut lines = Vec::with_capacity(2 + users_with_roles.len());

    // ---

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

    // ---

    for (user, roles) in users_with_roles {
        // ---

        let role_labels = roles
            .iter()
            .map(|role| role.to_string())
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

// ---

pub async fn user_exists(user: &str) -> Result<bool> {
    // ---

    let repo = create_app_user_repo();

    match repo.find_by_username(user).await {
        Ok(_) => Ok(true),
        Err(e) if is_not_found_error(&e) => Ok(false),
        Err(e) => Err(e).context("Failed to query user existence in database"),
    }
}

// ---

fn is_not_found_error(e: &anyhow::Error) -> bool {
    // ---

    // TODO: This needs to be adapted to your specific error handling
    // The original used Diesel's NotFound error - adjust for your implementation
    e.to_string().contains("not found") || e.to_string().contains("NotFound")
}

// ---

pub async fn digest_send(email: String, hours_since: i32) -> Result<()> {
    // ---

    let repo = create_crate_repo();
    let crates = repo.find_since(hours_since).await?;

    if !crates.is_empty() {
        // ---

        tracing::info!("Sending digest for {} crates", crates.len());
        println!("📧 Sending digest for {} crates to {}", crates.len(), email);

        let mailer = create_mailer()?;
        mailer
            .send_digest(&email, &crates)
            .await
            .map_err(|e| anyhow!("Failed to send new crates email: {:?}", e))?;

        println!("✅ Digest email sent successfully");
    } else {
        println!("📭 No new crates found in the last {} hours", hours_since);
    }

    Ok(())
}

// ---

pub async fn init_default_roles() -> Result<()> {
    // ---

    anyhow::bail!("init_default_roles is no longer supported")
}

// ---

#[cfg(test)]
mod tests {

    // TBD
}
