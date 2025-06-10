// src/rocket_routes/mod.rs
//! Rocket route mounting and API composition layer.
//!
//! This module defines and wires together all HTTP endpoints using Rocket,
//! mapping request routes to domain-layer traits and repository-backed handlers.
//!
//! It contains:
//! - Public API entrypoints (e.g., `/login`, `/crates`, `/rustaceans`)
//! - Rocket guards for authentication and authorization
//! - CORS setup and default fallback handlers
//!
//! It does *not* contain business logic or direct database access.
//! All logic is delegated to domain traits, ensuring separation of concerns.

mod authorization;
mod authors;
mod crates;
mod guards;
mod health;
mod support;

pub use support::{options, server_error, Cors};

pub use authorization::{login, me};

pub use crates::{
    // ---
    create_crate,
    delete_crate,
    get_crates,
    update_crate,
    view_crate,
};

pub use guards::{EditorUser, GuardedAppUser};

pub use health::health_endpoint;

pub use authors::{
    // ---
    create_rustacean,
    delete_rustacean,
    get_rustaceans,
    update_rustacean,
    view_rustacean,
};

use rocket::get;

/// HTML landing page for the root `/` path.
///
/// Displays a brief introduction and links to API endpoints.
#[get("/")]
pub fn index() -> rocket::response::content::RawHtml<&'static str> {
    rocket::response::content::RawHtml(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>CR8S API</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; }
            .api-link { color: #007acc; text-decoration: none; }
            .api-link:hover { text-decoration: underline; }
            .status { margin: 1rem 0; padding: 1rem; background: #f5f5f5; border-radius: 4px; }
        </style>
    </head>
    <body>
        <h1>🧪 CR8S API Server</h1>
        <p>Welcome to the CR8S (Crates) API - a Rust web service for managing crate and author information.</p>
        
        <div class="status">
            <h3>🔗 API Endpoints</h3>
            <ul>
                <li><a href="/cr8s/health" class="api-link">/cr8s/health</a> - Health check</li>
                <li><a href="/cr8s/login" class="api-link">/cr8s/login</a> - Authentication (POST)</li>
                <li><a href="/cr8s/rustaceans" class="api-link">/cr8s/rustaceans</a> - Authors (requires auth)</li>
                <li><a href="/cr8s/crates" class="api-link">/cr8s/crates</a> - Crates (requires auth)</li>
            </ul>
        </div>
        
        <p>📖 For documentation and setup instructions, visit the 
           <a href="https://github.com/johnbasrai/cr8s" class="api-link">GitHub repository</a>.</p>
    </body>
    </html>
    "#,
    )
}
