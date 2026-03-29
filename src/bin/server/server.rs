// src/bin/server/server.rs

//! Full Rocket server initialization and CLI-aware entrypoint logic.
//!
//! Handles:
//! - Database and cache setup
//! - CLI flag parsing and route inspection
//! - Rocket state management and route mounting

// Declare the modules we need directly in this file

use super::{
    //
    find_missing_state_types,
    find_unused_state_types,
    generate_route_state_markdown,
};
use crate::Cli;
use anyhow::{Context, Result};

macro_rules! init_servcies {
    () => {{
        tokio::try_join!(
            cr8s::domain::init_database_with_retry_from_env(),
            cr8s::domain::init_cache_with_retry_from_env()
        )
        .with_context(|| "Startup failed: check Redis/Postgres connection")
    }};
}

// ---

type Rocket = rocket::Rocket<rocket::Build>;

// ---
/// Launches the Rocket web server after initializing all infrastructure.
///
/// Responds to CLI flags for inspection (e.g., dump/check) but continues
/// to launch the app unless an error occurs.
pub async fn run() -> Result<(), anyhow::Error> {
    // --
    init_tracing();

    tracing::info!("✅ backend starting...");
    tracing::info!("Starting cr8s backend (Rocket + Redis + Postgres)...");

    let start = std::time::Instant::now();

    // Parse CLI args early to handle --help without database
    use clap::Parser;
    let cli = Cli::parse();

    // Always initialize services and build rocket
    init_servcies!()?;
    let rocket = build_rocket()?;

    // Process inspection flags if requested (but don't exit)
    if cli.check || cli.dump_state_traits || cli.output.is_some() {
        // ---
        handle_inspection_args(&cli, &rocket)?;
        // Continue execution instead of returning
    }

    // Always do the unused state check and start server
    check_unused_managed_state(&rocket);
    tracing::info!("Startup completed in {:?}", start.elapsed());
    rocket.launch().await?;

    Ok(())
}

/// Handles inspection-related CLI flags (`--check`, `--dump-state-traits`, `--output`).
///
/// If requested, this prints or saves a Markdown summary of route dependencies.
/// Does not terminate the server.
fn handle_inspection_args(cli: &Cli, rocket: &Rocket) -> Result<bool, anyhow::Error> {
    // ---
    if cli.check || cli.dump_state_traits || cli.output.is_some() {
        // ---
        let table = generate_route_state_markdown(rocket)?;

        if cli.dump_state_traits {
            println!("{table}");
        } else if let Some(path) = &cli.output {
            std::fs::write(path, table)?;
            println!("✅ Route-trait table written to {}", path.display());
        }

        if cli.check {
            check_and_dump_statistics(rocket);
        }

        return Ok(true);
    }

    Ok(false)
}

// ---

// Define a debug macro
macro_rules! debug_managed_type {
    ($name:expr, $value:expr) => {
        tracing::debug!("  - {}: {}", $name, std::any::type_name_of_val($value));
    };
}

/// Builds the Rocket instance with all managed state and routes mounted.
///
/// Injects repositories, password hasher, cache context, and health services.
/// Mounts all HTTP endpoints and attaches CORS.
fn build_rocket() -> Result<rocket::Rocket<rocket::Build>, anyhow::Error> {
    // ---

    let app_user_repo = cr8s::domain::create_app_user_repo();
    let crate_repo = cr8s::domain::create_crate_repo();
    let author_repo = cr8s::domain::create_author_repo();
    let cache_context = cr8s::domain::create_cache_context();
    let password_hasher = cr8s::domain::create_password_hasher()?;
    let health_service = cr8s::domain::create_cache_health_service()?;

    // DEBUG: Log what we're managing
    tracing::info!("🔧 Managing state types:");
    debug_managed_type!("AppUserRepo", &app_user_repo);
    debug_managed_type!("CrateRepo", &crate_repo);
    debug_managed_type!("AuthorRepo", &author_repo);
    debug_managed_type!("CacheContext", &cache_context);
    debug_managed_type!("PasswordHasher", &password_hasher);
    debug_managed_type!("HealthService", &health_service);

    Ok(rocket::build()
        .manage(app_user_repo)
        .manage(crate_repo)
        .manage(author_repo)
        .manage(cache_context)
        .manage(password_hasher)
        .manage(health_service)
        .mount("/", rocket::routes![cr8s::rocket_routes::index])
        .mount(
            "/cr8s",
            rocket::routes![
                cr8s::rocket_routes::health_endpoint,
                cr8s::rocket_routes::options,
                cr8s::rocket_routes::me,
                cr8s::rocket_routes::login,
                cr8s::rocket_routes::get_rustaceans,
                cr8s::rocket_routes::view_rustacean,
                cr8s::rocket_routes::create_rustacean,
                cr8s::rocket_routes::update_rustacean,
                cr8s::rocket_routes::delete_rustacean,
                cr8s::rocket_routes::get_crates,
                cr8s::rocket_routes::view_crate,
                cr8s::rocket_routes::create_crate,
                cr8s::rocket_routes::update_crate,
                cr8s::rocket_routes::delete_crate,
            ],
        )
        .attach(cr8s::rocket_routes::Cors))
}

// ---

/// Initializes tracing/logging with `RUST_LOG` support.
fn init_tracing() {
    // ---

    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).with_target(true).init();
}

// ---

/// Check for unused/missing managed state types and log warnings/errors
fn check_unused_managed_state(rocket: &Rocket) {
    // ---

    let unused = find_unused_state_types(rocket);
    if !unused.is_empty() {
        tracing::warn!("⚠️  Unused managed state types: {:?}", unused);
    }

    // ---

    let missing = find_missing_state_types(rocket);
    if !missing.is_empty() {
        tracing::error!("❌ Missing managed state types: {:?}", missing);
    }
}

// ---

/// Analyze state management and print statistics (exits with error if issues found)
fn check_and_dump_statistics(rocket: &Rocket) {
    // ---

    let unused = find_unused_state_types(rocket);
    let missing = find_missing_state_types(rocket);

    // ---

    println!("📊 State Management Statistics:");
    println!("  - Unused managed types: {}", unused.len());
    println!("  - Missing required types: {}", missing.len());

    // ---

    if !unused.is_empty() {
        println!("⚠️  Unused: {unused:?}");
    }

    if !missing.is_empty() {
        println!("❌ Missing: {missing:?}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_unused_managed_state_logs_appropriately() -> Result<()> {
        // ---

        let rocket = rocket::build();

        // This function should not panic with any rocket instance
        check_unused_managed_state(&rocket);

        // Note: We can't easily test the tracing output without complex setup
        // But we can ensure it doesn't crash
        Ok(())
    }
}
