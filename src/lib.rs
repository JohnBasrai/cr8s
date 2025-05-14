pub mod auth;
pub mod commands;
mod mail;
pub mod models;
mod repositories;
pub mod rocket_routes;
mod schema;

#[cfg(test)]
mod tests {
    mod authorization;
    mod crates;
    mod rustaceans;
    pub mod test_utils;
}
