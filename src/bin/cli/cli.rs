// src/bin/cli/cli.rs
// CLI argument parsing and structure definitions

use clap::{Parser, Subcommand};
use cr8s::domain::RoleCode;

// ---

#[derive(Parser)]
#[command(
    name = "cr8s-cli",
    version,
    about = "CR8S CLI tool for managing users, authors, and crates",
    author = "John Basrai <john@basrai.dev>"
)]
pub struct Cli {
    // ---
    #[command(subcommand)]
    pub command: Commands,
}

// ---

#[derive(Subcommand)]
pub enum Commands {
    // ---
    /// Create a new user with specified roles
    CreateUser {
        /// Username for the new user
        #[arg(short, long)]
        username: String,

        /// Password for the new user  
        #[arg(short, long)]
        password: String,

        /// Roles to assign (case insensitive)
        /// Valid roles: Admin, Editor, Viewer (or a/e/v shortcuts)
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<CliRoleCode>,
    },

    /// Delete a user by ID
    DeleteUser {
        /// User ID to delete
        #[arg(allow_hyphen_values = true)]
        user_id: i32,
    },

    /// Delete a user by username
    DeleteUserByName {
        /// Username to delete
        username: String,
    },

    /// List all users with their roles
    ListUsers,

    /// Check if a user exists
    UserExists {
        /// Username to check
        username: String,
    },

    /// Send digest email with recent crates
    DigestSend {
        /// Email address to send digest to
        #[arg(short, long)]
        email: String,

        /// Hours since to include crates (default: 24)
        #[arg(long, default_value = "24")]
        hours_since: i32,
    },

    /// Initialize default roles in the system
    InitDefaultRoles,
}

// ---

// Clap-compatible wrapper for RoleCode (domain type can't have clap derives)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliRoleCode {
    Admin,
    Editor,
    Viewer,
}

// Add this implementation back
impl From<CliRoleCode> for RoleCode {
    fn from(cli_role: CliRoleCode) -> Self {
        match cli_role {
            CliRoleCode::Admin => RoleCode::Admin,
            CliRoleCode::Editor => RoleCode::Editor,
            CliRoleCode::Viewer => RoleCode::Viewer,
        }
    }
}

impl std::str::FromStr for CliRoleCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "admin" | "a" => Ok(CliRoleCode::Admin),
            "editor" | "edit" | "e" => Ok(CliRoleCode::Editor),
            "viewer" | "view" | "v" => Ok(CliRoleCode::Viewer),
            _ => Err(format!(
                "Invalid role: '{}'. Valid roles: Admin, Editor, Viewer (or a/e/v shortcuts)",
                s
            )),
        }
    }
}

impl std::fmt::Display for CliRoleCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliRoleCode::Admin => write!(f, "Admin"),
            CliRoleCode::Editor => write!(f, "Editor"),
            CliRoleCode::Viewer => write!(f, "Viewer"),
        }
    }
}

// ---

#[cfg(test)]
mod tests {
    //---
    use super::*;
    use anyhow::{ensure, Result};
    use clap::Parser;

    // ---

    #[test]
    fn test_create_user_basic_one_role() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "testuser",
            "--password",
            "secret123",
            "--roles",
            "admin",
        ]);

        match args.command {
            Commands::CreateUser {
                username,
                password,
                roles,
            } => {
                ensure!(username == "testuser");
                ensure!(password == "secret123");
                ensure!(roles.len() == 1);
                ensure!(roles[0] == CliRoleCode::Admin);
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_create_user_multiple_roles_comma_separated() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "multiuser",
            "--password",
            "complex_pass",
            "--roles",
            "admin,editor,viewer",
        ]);

        match args.command {
            Commands::CreateUser {
                username,
                password,
                roles,
            } => {
                ensure!(username == "multiuser");
                ensure!(password == "complex_pass");
                ensure!(roles.len() == 3);
                ensure!(roles.contains(&CliRoleCode::Admin));
                ensure!(roles.contains(&CliRoleCode::Editor));
                ensure!(roles.contains(&CliRoleCode::Viewer));
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_create_user_no_roles() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "noroles",
            "--password",
            "pass123",
        ]);

        match args.command {
            Commands::CreateUser {
                username,
                password,
                roles,
            } => {
                ensure!(username == "noroles");
                ensure!(password == "pass123");
                ensure!(roles.is_empty());
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_delete_user_by_id() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "delete-user", "42"]);

        match args.command {
            Commands::DeleteUser { user_id } => {
                ensure!(user_id == 42);
            }
            _ => anyhow::bail!("Expected DeleteUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_delete_user_by_name() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "delete-user-by-name", "olduser"]);

        match args.command {
            Commands::DeleteUserByName { username } => {
                ensure!(username == "olduser");
            }
            _ => anyhow::bail!("Expected DeleteUserByName command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_list_users() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "list-users"]);

        match args.command {
            Commands::ListUsers => {
                // No parameters to check, just ensure correct variant
            }
            _ => anyhow::bail!("Expected ListUsers command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_user_exists() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "user-exists", "checkme"]);

        match args.command {
            Commands::UserExists { username } => {
                ensure!(username == "checkme");
            }
            _ => anyhow::bail!("Expected UserExists command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_digest_send_basic() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "digest-send", "--email", "test@example.com"]);

        match args.command {
            Commands::DigestSend { email, hours_since } => {
                ensure!(email == "test@example.com");
                ensure!(hours_since == 24); // Default value
            }
            _ => anyhow::bail!("Expected DigestSend command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_digest_send_custom_hours() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "digest-send",
            "--email",
            "weekly@example.com",
            "--hours-since",
            "168",
        ]);

        match args.command {
            Commands::DigestSend { email, hours_since } => {
                ensure!(email == "weekly@example.com");
                ensure!(hours_since == 168); // 1 week
            }
            _ => anyhow::bail!("Expected DigestSend command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_init_default_roles() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "init-default-roles"]);

        match args.command {
            Commands::InitDefaultRoles => {
                // No parameters to check
            }
            _ => anyhow::bail!("Expected InitDefaultRoles command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_role_code_conversion() -> Result<()> {
        // ---

        let admin: RoleCode = CliRoleCode::Admin.into();
        let editor: RoleCode = CliRoleCode::Editor.into();
        let viewer: RoleCode = CliRoleCode::Viewer.into();

        // Can't directly compare RoleCode values without Debug/PartialEq
        // But we can ensure conversion doesn't panic
        let _admin_str = format!("{:?}", admin);
        let _editor_str = format!("{:?}", editor);
        let _viewer_str = format!("{:?}", viewer);

        Ok(())
    }

    // ---

    #[test]
    fn test_role_code_cloning_and_equality() -> Result<()> {
        // ---

        let role1 = CliRoleCode::Admin;
        let role2 = role1.clone();
        let role3 = CliRoleCode::Editor;

        ensure!(role1 == role2);
        ensure!(role1 != role3);
        ensure!(role2 != role3);

        Ok(())
    }

    // ---

    // Edge cases and error conditions

    #[test]
    fn test_empty_username_allowed() -> Result<()> {
        // ---

        // CLI parsing allows empty strings, business logic should validate
        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "",
            "--password",
            "pass",
        ]);

        match args.command {
            Commands::CreateUser { username, .. } => {
                ensure!(username.is_empty());
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_negative_user_id() -> Result<()> {
        // ---

        let args = Cli::parse_from(["cr8s-cli", "delete-user", "-5"]);

        match args.command {
            Commands::DeleteUser { user_id } => {
                ensure!(user_id == -5);
            }
            _ => anyhow::bail!("Expected DeleteUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_zero_hours_since() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "digest-send",
            "--email",
            "immediate@example.com",
            "--hours-since",
            "0",
        ]);

        match args.command {
            Commands::DigestSend { email, hours_since } => {
                ensure!(email == "immediate@example.com");
                ensure!(hours_since == 0);
            }
            _ => anyhow::bail!("Expected DigestSend command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_large_hours_since() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "digest-send",
            "--email",
            "archive@example.com",
            "--hours-since",
            "8760", // 1 year
        ]);

        match args.command {
            Commands::DigestSend { email, hours_since } => {
                ensure!(email == "archive@example.com");
                ensure!(hours_since == 8760);
            }
            _ => anyhow::bail!("Expected DigestSend command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_special_characters_in_username() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "user.name+test@domain",
            "--password",
            "p@ssw0rd!",
        ]);

        match args.command {
            Commands::CreateUser {
                username, password, ..
            } => {
                ensure!(username == "user.name+test@domain");
                ensure!(password == "p@ssw0rd!");
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_unicode_in_username() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            "用戶名",
            "--password",
            "パスワード",
        ]);

        match args.command {
            Commands::CreateUser {
                username, password, ..
            } => {
                ensure!(username == "用戶名");
                ensure!(password == "パスワード");
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    // ---

    #[test]
    fn test_very_long_input() -> Result<()> {
        // ---

        let long_username = "a".repeat(1000);
        let long_password = "b".repeat(1000);

        let args = Cli::parse_from([
            "cr8s-cli",
            "create-user",
            "--username",
            &long_username,
            "--password",
            &long_password,
        ]);

        match args.command {
            Commands::CreateUser {
                username, password, ..
            } => {
                ensure!(username.len() == 1000);
                ensure!(password.len() == 1000);
                ensure!(username.chars().all(|c| c == 'a'));
                ensure!(password.chars().all(|c| c == 'b'));
            }
            _ => anyhow::bail!("Expected CreateUser command"),
        }

        Ok(())
    }

    #[test]
    fn test_role_case_insensitive_parsing() -> Result<()> {
        // ---
        let admin = "ADMIN"
            .parse::<CliRoleCode>()
            .map_err(|e| anyhow::anyhow!(e))?;
        let editor = "EdItOr"
            .parse::<CliRoleCode>()
            .map_err(|e| anyhow::anyhow!(e))?;
        let viewer = "viewer"
            .parse::<CliRoleCode>()
            .map_err(|e| anyhow::anyhow!(e))?;

        ensure!(admin == CliRoleCode::Admin);
        ensure!(editor == CliRoleCode::Editor);
        ensure!(viewer == CliRoleCode::Viewer);

        Ok(())
    }

    #[test]
    fn test_invalid_role_parsing() {
        // ---
        let result = "invalid_role".parse::<CliRoleCode>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid role"));
    }

    // ---

    // Note: These tests would fail at parse time, so we can't test them easily
    // without custom error handling:
    // - Invalid role names (clap handles this)
    // - Missing required arguments (clap handles this)
    // - Invalid integer formats (clap handles this)

    // These would require using try_parse_from instead of parse_from
    // and checking for Err results, but that's more complex testing
}
