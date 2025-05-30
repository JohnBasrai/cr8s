// src/bin/server/main.rs

mod diagnostics;
mod server;
mod server_cli_args;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    server::run().await
}
