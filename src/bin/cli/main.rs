// src/bin/cli/main.rs

// ---

// Sibling module declarations
mod cli;
mod commands;

// ---

// Internal-only imports (no pub use needed - binary has no external consumers)
use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use commands::{
    //
    create_user,
    delete_user_by_id,
    delete_user_by_username,
    digest_send,
    list_users_formatted,
    user_exists,
};
use cr8s::domain::{
    //
    init_cache_with_retry_from_env,
    init_database_with_retry_from_env,
    // -- Call into dab module to initialize cr8s schema and default roles
    load_schema_from_sql_file,
};

// ---

#[tokio::main]
async fn main() -> Result<()> {
    // ---

    init_tracing();

    tracing::info!("🚀 CR8S CLI starting...");

    // ---

    // Parse before initializing of database/redis which would require env settings.  This
    // allows us to parse the --help arg w/o requiring database url to be set.
    let cli = Cli::parse();

    // ---

    // Initialize infrastructure first (following clean architecture)
    tokio::try_join!(
        init_database_with_retry_from_env(),
        init_cache_with_retry_from_env()
    )
    .context("Failed to initialize database and cache")?;

    match cli.command {
        // ---
        Commands::CreateUser {
            username,
            password,
            roles,
        } => {
            // ---

            let role_codes = roles.into_iter().map(|r| r.into()).collect();
            create_user(username, password, role_codes).await
        }

        Commands::DeleteUser { user_id } => delete_user_by_id(user_id).await,
        Commands::DeleteUserByName { username } => delete_user_by_username(&username).await,
        Commands::ListUsers => {
            // ---

            let lines = list_users_formatted().await?;
            for line in lines {
                println!("{}", line);
            }
            Ok(())
        }

        Commands::UserExists { username } => {
            // ---

            let exists = user_exists(&username).await?;
            if exists {
                println!("✅ User '{}' exists", username);
            } else {
                println!("❌ User '{}' does not exist", username);
            }
            Ok(())
        }

        Commands::DigestSend { email, hours_since } => digest_send(email, hours_since).await,
        Commands::LoadSchema => load_schema_from_sql_file().await,
    }
}

// ---

fn init_tracing() {
    // ---

    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).with_target(true).init();
}
