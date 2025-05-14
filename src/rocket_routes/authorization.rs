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
    // ---
    tracing::info!("🔐 Login attempt for: {}", credentials.username);

    let user = match UserRepository::find_by_username(&mut db, &credentials.username).await {
        Ok(user) => {
            tracing::info!("👤 Found user: {}", user.username);
            user
        }
        Err(diesel::result::Error::NotFound) => {
            tracing::warn!("🚫 User not found: {}", credentials.username);
            return Err(Custom(
                Status::Unauthorized,
                json!("error: Invalid credentials"),
            ));
        }
        Err(e) => return Err(server_error(e)),
    };

    let creds = credentials.into_inner();
    match authorize_user(&user, creds.clone()) {
        Ok(session_id) => {
            tracing::info!("✅ Password accepted for: {}", user.username);
            cache
                .set_ex::<String, i32, ()>(format!("sessions/{}", session_id), user.id, 3 * 60 * 60)
                .await
                .map_err(server_error)?;

            Ok(json!({ "token": session_id }))
        }
        Err(_) => {
            tracing::warn!("❌ Password verification failed for: {}", user.username);
            Err(Custom(
                Status::Unauthorized,
                json!("error: Wrong credentials"),
            ))
        }
    }
}

#[rocket::get("/me")]
pub fn me(user: User) -> Value {
    tracing::info!("✅ Authenticated user: {:?}", user);
    json!(user)
}
