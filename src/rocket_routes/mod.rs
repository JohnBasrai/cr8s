// src/rocket_routes/mod.rs
//! Rocket route mounting and API composition layer.
//!
//! This module wires together domain traits and repository impls for routing.
//! It does not contain business logic, only HTTP entrypoints and wiring.

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
