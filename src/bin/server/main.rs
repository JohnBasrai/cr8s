// src/bin/server/main.rs
//! Entry point for the `cr8s` Rocket server binary.
//!
//! Delegates startup logic to `server::run`, which parses CLI flags,
//! initializes services (Postgres, Redis), builds Rocket, and launches the app.
//!
//! CLI options for inspection (`--dump-state-traits`, `--check`, `--output`) are parsed here.

//
// Bring all submodules into scope
//
mod diagnostics;
mod server;
mod server_cli_args;

//
// Internal-only exports (sibling access within this module)
//
use diagnostics::{
    //
    find_missing_state_types,
    find_unused_state_types,
    generate_route_state_markdown,
};
use server_cli_args::Cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    server::run().await
}
