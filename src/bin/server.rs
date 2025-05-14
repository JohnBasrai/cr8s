extern crate cr8s;

use rocket_db_pools::Database;
//use tracing_subscriber::{fmt, EnvFilter};

#[rocket::main]
async fn main() {
    // ---
    init_tracing(); // ✅
    tracing::info!("✅ backend starting...");

    let _ = rocket::build()
        .mount(
            "/",
            rocket::routes![
                cr8s::rocket_routes::options,
                cr8s::rocket_routes::authorization::me,
                cr8s::rocket_routes::authorization::login,
                cr8s::rocket_routes::rustaceans::get_rustaceans,
                cr8s::rocket_routes::rustaceans::view_rustacean,
                cr8s::rocket_routes::rustaceans::create_rustacean,
                cr8s::rocket_routes::rustaceans::update_rustacean,
                cr8s::rocket_routes::rustaceans::delete_rustacean,
                cr8s::rocket_routes::crates::get_crates,
                cr8s::rocket_routes::crates::view_crate,
                cr8s::rocket_routes::crates::create_crate,
                cr8s::rocket_routes::crates::update_crate,
                cr8s::rocket_routes::crates::delete_crate,
            ],
        )
        .attach(cr8s::rocket_routes::Cors)
        .attach(cr8s::rocket_routes::CacheConn::init())
        .attach(cr8s::rocket_routes::DbConn::init())
        .launch()
        .await;
}

fn init_tracing() {
    // ---
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // or "debug"

    fmt().with_env_filter(filter).with_target(true).init();
}
