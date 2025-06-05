//! Integration tests for cr8s CLI commands
//!
//! These tests run against a live docker compose stack with:
//! - postgres database
//! - redis cache  
//! - cr8s server
//! - cr8s cli (via docker compose)
//!
//! Prerequisites:
//! 1. Build dev images: `./scripts/build-images.sh --dev`
//! 2. Set up environment: `./scripts/dev-test-setup.sh`
//! 3. Run tests: `cargo test --test cli_integration`
//!
//! Tests run sequentially to avoid database conflicts.

use anyhow::{ensure, Context, Result};
use std::process::{Command, Stdio};
use std::str;

/// Base command builder for cr8s CLI via docker compose
fn cli_command() -> Command {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "run", "--rm", "cli"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

/// Helper to run CLI command and return output
async fn run_cli_command(args: &[&str]) -> Result<(String, String, i32)> {
    let output = cli_command()
        .args(args)
        .output()
        .context("Failed to execute CLI command")?;

    let stdout = str::from_utf8(&output.stdout)
        .context("Failed to parse stdout as UTF-8")?
        .to_string();

    let stderr = str::from_utf8(&output.stderr)
        .context("Failed to parse stderr as UTF-8")?
        .to_string();

    Ok((stdout, stderr, output.status.code().unwrap_or(-1)))
}

/// Helper to assert command succeeded
async fn assert_cli_success(args: &[&str]) -> Result<String> {
    let (stdout, stderr, code) = run_cli_command(args).await?;

    ensure!(
        code == 0,
        "CLI command failed with code {}\nArgs: {:?}\nStdout: {}\nStderr: {}",
        code,
        args,
        stdout,
        stderr
    );

    Ok(stdout)
}

/// Helper to assert command failed with specific code
async fn assert_cli_failure(args: &[&str], expected_code: i32) -> Result<String> {
    let (stdout, stderr, code) = run_cli_command(args).await?;

    ensure!(
        code == expected_code,
        "Expected CLI command to fail with code {}, got {}\nArgs: {:?}\nStdout: {}\nStderr: {}",
        expected_code,
        code,
        args,
        stdout,
        stderr
    );

    Ok(stderr)
}

#[tokio::test]
async fn test_complete_cli_workflow() -> Result<()> {
    println!("🚀 Starting complete CLI integration test");

    // Step 1: Set up clean database
    // Note: Success means CLI executed without error; Step 3 will verify tables exist
    println!("📋 Step 1: Loading schema");
    assert_cli_success(&["load-schema"]).await?;

    // Step 2: User management workflow
    // Note: Success means CLI executed without error; Step 3 will verify users were created
    println!("👤 Step 2: Creating test users");
    assert_cli_success(&[
        "create-user",
        "--username",
        "test-user-1",
        "--password",
        "pass123",
        "--roles",
        "admin",
    ])
    .await?;
    assert_cli_success(&[
        "create-user",
        "--username",
        "test-user-2",
        "--password",
        "pass456",
        "--roles",
        "editor",
    ])
    .await?;

    // Step 3: Verify users exist
    println!("📋 Step 3: Listing users");
    let user_list = assert_cli_success(&["list-users"]).await?;
    ensure!(user_list.contains("test-user-1"), "test-user-1 not found");
    ensure!(user_list.contains("test-user-2"), "test-user-2 not found");

    // Step 4: Test user existence checks
    println!("🔍 Step 4: Testing user existence");
    assert_cli_success(&["user-exists", "test-user-1"]).await?;
    assert_cli_failure(&["user-exists", "junk-user"], 1).await?;

    // Step 5: Delete users and verify
    println!("🗑️  Step 5: Testing user deletion");
    assert_cli_success(&["delete-user-by-name", "test-user-1"]).await?;

    // Step 5b: Verify deleted user doesn't exist
    println!("🔍 Step 5b: Verifying deleted user doesn't exist");
    assert_cli_failure(&["user-exists", "test-user-1"], 1).await?;

    // Verify user-1 is gone, user-2 still exists
    let user_list = assert_cli_success(&["list-users"]).await?;
    ensure!(
        !user_list.contains("test-user-1"),
        "test-user-1 still exists"
    );
    ensure!(user_list.contains("test-user-2"), "test-user-2 missing");

    // Step 6: Test role shortcuts
    println!("🎭 Step 6: Testing role shortcuts");
    assert_cli_success(&[
        "create-user",
        "--username",
        "short-admin",
        "--password",
        "pass",
        "--roles",
        "a",
    ])
    .await?;

    // Verify the short-admin user was created
    let user_list = assert_cli_success(&["list-users"]).await?;
    ensure!(
        user_list.contains("short-admin"),
        "short-admin user not created"
    );

    // Step 7: Test error scenarios
    println!("❌ Step 7: Testing error scenarios");
    assert_cli_failure(
        &[
            "create-user",
            "--username",
            "bad-role-user",
            "--password",
            "pass",
            "--roles",
            "invalid",
        ],
        2,
    )
    .await?;

    // Step 8: Test digest (email functionality)
    // Note: Exit code 0 means actual SMTP delivery succeeded (not just a stub)
    // Requires SMTP_HOST, SMTP_USERNAME, SMTP_PASSWORD env vars
    // See: cr8s/src/mail/html_mailer.rs
    println!("📧 Step 8: Testing digest (optional)");
    let (_stdout, stderr, code) =
        run_cli_command(&["digest-send", "--email", "test@example.com"]).await?;
    if code == 0 {
        println!("✅ Digest sent successfully");
    } else {
        println!(
            "⚠️  Digest failed (expected if email not configured): {}",
            stderr.trim()
        );
    }

    // Step 9: Cleanup
    println!("🧹 Step 9: Final cleanup");
    assert_cli_success(&["delete-user-by-name", "test-user-2"]).await?;
    assert_cli_success(&["delete-user-by-name", "short-admin"]).await?;

    // Step 10: Verify cleanup worked
    println!("🔍 Step 10: Verifying all test users removed");
    let final_user_list = assert_cli_success(&["list-users"]).await?;
    ensure!(
        !final_user_list.contains("test-user"),
        "Test users still exist after cleanup"
    );

    println!("🎉 Complete CLI workflow test passed!");
    Ok(())
}
