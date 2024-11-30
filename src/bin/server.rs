extern crate cr8s;

use rocket_db_pools::Database;

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![
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
        ])
        .attach(cr8s::rocket_routes::Cors)
        .attach(cr8s::rocket_routes::CacheConn::init())
        .attach(cr8s::rocket_routes::DbConn::init())
        .launch()
        .await;
}
