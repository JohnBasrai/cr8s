// src/bin/server/main.rs

mod server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    server::run().await
}
