// src/bin/server/main.rs

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
