// src/bin/server/server.rs
// Server implementation logic

// Declare the modules we need directly in this file
mod diagnostics;
mod server_cli_args;

use crate::diagnostics::{
    //
    find_missing_state_types,
    find_unused_state_types,
    generate_route_state_markdown,
};
use crate::server_cli_args::Cli;
use anyhow::{Context, Result};
use clap::Parser;

// ---

type Rocket = rocket::Rocket<rocket::Build>;

// ---

pub async fn run() -> Result<(), anyhow::Error> {
    // ---

    init_tracing();

    tracing::info!("✅ backend starting...");
    tracing::info!("Starting cr8s backend (Rocket + Redis + Postgres)...");

    let start = std::time::Instant::now();

    // ---

    let rocket = build_rocket()?;

    if parse_and_handle_args(&rocket)? {
        return Ok(());
    }

    check_unused_managed_state(&rocket);

    // ---

    tokio::try_join!(
        cr8s::domain::init_database_with_retry_from_env(),
        cr8s::domain::init_cache_with_retry_from_env()
    )
    .with_context(|| "Startup failed: check Redis/Postgres connection")?;

    // ---

    tracing::info!("Startup completed in {:?}", start.elapsed());
    rocket.launch().await?;
    Ok(())
}

// ---

/// Build the rocket with all its fairings attached.
fn build_rocket() -> Result<rocket::Rocket<rocket::Build>, anyhow::Error> {
    // ---

    Ok(rocket::build()
        .manage(cr8s::domain::create_app_user_repo())
        .manage(cr8s::domain::create_crate_repo())
        .manage(cr8s::domain::create_author_repo())
        .manage(cr8s::domain::create_cache_context())
        .manage(cr8s::domain::create_password_hasher()?)
        .manage(cr8s::domain::create_cache_health_service()?)
        .mount(
            "/",
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

fn init_tracing() {
    // ---

    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt().with_env_filter(filter).with_target(true).init();
}

// ---

fn parse_and_handle_args(rocket: &Rocket) -> Result<bool, anyhow::Error> {
    // ---

    let cli = Cli::parse();

    if cli.check || cli.dump_state_traits || cli.output.is_some() {
        // ---

        let table = generate_route_state_markdown(rocket)?;

        if cli.dump_state_traits {
            println!("{}", table);
        } else if let Some(path) = cli.output {
            // ---

            std::fs::write(&path, table)?;
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
        println!("⚠️  Unused: {:?}", unused);
    }

    if !missing.is_empty() {
        println!("❌ Missing: {:?}", missing);
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
