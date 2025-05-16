use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use cr8s::commands;
use cr8s::models::RoleCode;
//use diesel_derive_enum::DbEnum;

#[derive(Parser)]
#[command(name = "cli", version, about = "User management")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Users {
        #[command(subcommand)]
        subcommand: UserSubcommand,
    },
    Roles {
        #[command(subcommand)]
        subcommand: RoleSubcommand,
    },
}

// ┌──────────────────────────────────────────────┐
// │ CLI Usage Examples (developer reference)     │
// └──────────────────────────────────────────────┘
// cli users create alice password123 admin editor
// cli users delete 42
// cli users delete alice
// cli users list

#[derive(clap::Parser, Debug)]
pub enum UserSubcommand {
    /// Create a new user with one or more roles
    Create {
        /// The username to create
        username: String,

        /// The password for the new user
        password: String,

        /// One or more roles to assign (e.g. admin editor viewer)
        #[arg(required = true, value_enum)]
        roles: Vec<RoleCode>,
    },

    /// Delete a user by ID or username
    Delete {
        /// User ID or username
        id_or_username: String,
    },

    /// List all users with their assigned roles
    List,
}

#[derive(clap::Parser, Debug)]
pub enum RoleSubcommand {
    /// Seed default roles into the database (admin, editor, viewer)
    InitDefaults,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ---

    // Set up tracing subscriber for CLI logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Users { subcommand } => match subcommand {
            UserSubcommand::Create {
                username,
                password,
                roles,
            } => Cli::create_user(username, password, roles).await,
            UserSubcommand::Delete { id_or_username } => Cli::delete_user(&id_or_username).await,
            UserSubcommand::List => Cli::list_users().await,
        },

        Command::Roles { subcommand } => match subcommand {
            RoleSubcommand::InitDefaults => Cli::init_default_roles().await,
        },
    }
}

impl Cli {
    // ---
    async fn init_default_roles() -> Result<()> {
        // ---
        commands::init_default_roles().await
    }

    async fn create_user(username: String, password: String, roles: Vec<RoleCode>) -> Result<()> {
        // ---
        let user_exists = commands::user_exists(&username).await?;

        if user_exists {
            return Err(anyhow!("User '{}' already exists.", username));
        }

        commands::create_user(username.clone(), password, roles)
            .await
            .context("Failed to create or retrieve user")?;

        tracing::info!("✅ User created: {}", username);
        Ok(())
    }

    async fn list_users() -> Result<()> {
        // ---
        let users = commands::list_users_formatted()
            .await
            .with_context(|| "Failed to list users")?;

        for u in users {
            tracing::info!("{u}");
        }

        Ok(())
    }

    async fn delete_user(id_or_username: &str) -> Result<()> {
        // ---
        let result = if let Ok(id) = id_or_username.parse::<i32>() {
            commands::delete_user_by_id(id).await
        } else {
            commands::delete_user_by_username(id_or_username).await
        };

        result.with_context(|| "Failed to delete user")?;

        tracing::info!("User deleted.");
        Ok(())
    }
} // impl Cli
