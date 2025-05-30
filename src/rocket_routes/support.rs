// src/rocket_routes/support.rs
//! Rocket support utilities: error handler, CORS fairing, and OPTIONS fallback.

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use rocket::{Request, Response};

/// Shared error handler that logs the full error and returns a clean JSON message.
pub fn server_error<E: std::fmt::Display>(e: E) -> Custom<Value> {
    rocket::error!("🚨 Internal server error: {}", e);
    Custom(
        Status::InternalServerError,
        json!({ "error": "Something went wrong" }),
    )
}

/// Generic fallback handler to allow CORS preflight requests.
#[rocket::options("/<_route_args..>")]
pub fn options(_route_args: Option<std::path::PathBuf>) {
    // Just to add CORS headers via the fairing.
}

/// Rocket fairing to inject permissive CORS headers into all responses.
pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Append CORS headers in responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_raw_header("Access-Control-Allow-Origin", "*");
        res.set_raw_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE");
        res.set_raw_header("Access-Control-Allow-Headers", "*");
        res.set_raw_header("Access-Control-Allow-Credentials", "true");
    }
}
