use std::error::Error;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use rocket_db_pools::Connection;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;

use crate::models::{User, RoleCode};
use crate::repositories::{UserRepository, RoleRepository};

pub mod authorization;
pub mod crates;
pub mod rustaceans;

#[derive(rocket_db_pools::Database)]
#[database("postgres")]
pub struct DbConn(rocket_db_pools::diesel::PgPool);

#[derive(rocket_db_pools::Database)]
#[database("redis")]
pub struct CacheConn(rocket_db_pools::deadpool_redis::Pool);

pub fn server_error(e: Box<dyn Error>) -> Custom<Value> {
    rocket::error!("{}", e);
    Custom(Status::InternalServerError, json!("Error"))
}

#[rocket::options("/<_route_args..>")]
pub fn options(_route_args: Option<std::path::PathBuf>) {
    // Just to add CORS header via the fairing.
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Append CORS headers in responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_raw_header("Access-Control-Allow-Origin", "*");
        res.set_raw_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE");
        res.set_raw_header("Access-Control-Allow-Headers", "*");
        res.set_raw_header("Access-Control-Allow-Credentials", "true");
    }
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Authorization: Bearer SESSION_ID_128_CHARACTERS_LONG
        let session_header = req.headers().get_one("Authorization")
            .map(|v| v.split_whitespace().collect::<Vec<_>>())
            .filter(|v| v.len() == 2 && v[0] == "Bearer");
        if let Some(header_value) = session_header {
            let mut cache = req.guard::<Connection<CacheConn>>().await
                .expect("Can not connect to Redis in request guard");
            let mut db = req.guard::<Connection<DbConn>>().await
                .expect("Can not connect to Postgres in request guard");

            let result = cache.get::<String, i32>(format!("sessions/{}", header_value[1])).await;
            if let Ok(user_id) = result {
                if let Ok(user) = UserRepository::find(&mut db, user_id).await {
                    return Outcome::Success(user);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

pub struct EditorUser(User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EditorUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = req.guard::<User>().await
            .expect("Cannot retrieve current logged in user");
        let mut db = req.guard::<Connection<DbConn>>().await
            .expect("Can not connect to Postgres in request guard");

        if let Ok(roles) = RoleRepository::find_by_user(&mut db, &user).await {
            rocket::info!("Roles assigned are {:?}", roles);
            let is_editor = roles.iter().any(|r| match r.code {
                RoleCode::Admin => true,
                RoleCode::Editor => true,
                _ => false
            });
            rocket::info!("Is editor is {:?}", is_editor);

            if is_editor {
                return Outcome::Success(EditorUser(user));
            }
        }


        Outcome::Error((Status::Unauthorized, ()))
    }
}
