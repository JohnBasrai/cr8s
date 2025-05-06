use crate::auth::{authorize_user, Credentials};
use crate::models::User;
use crate::repositories::UserRepository;
use crate::rocket_routes::{server_error, CacheConn, DbConn};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

#[rocket::post("/login", format = "json", data = "<credentials>")]
pub async fn login(
    mut db: Connection<DbConn>,
    mut cache: Connection<CacheConn>,
    credentials: Json<Credentials>,
) -> Result<Value, Custom<Value>> {
    let user = UserRepository::find_by_username(&mut db, &credentials.username)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                rocket::error!("{}", e);
                Custom(Status::Unauthorized, json!("error: Invalid credentials"))
            }
            _ => server_error(e),
        })?;

    let session_id = authorize_user(&user, credentials.into_inner())
        .map_err(|_| Custom(Status::Unauthorized, json!("error: Wrong credentials")))?;

    cache
        .set_ex::<String, i32, ()>(format!("sessions/{}", session_id), user.id, 3 * 60 * 60)
        .await
        .map_err(server_error)?;

    Ok(json!({
        "token": session_id,
    }))
}

#[rocket::get("/me")]
pub fn me(user: User) -> Value {
    json!(user)
}
