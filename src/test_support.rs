use crate::models::{NewUser, User};
use crate::schema::users;
use anyhow::{Context, Result};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl as AsyncRunQueryDsl; // Aliased for disambiguation

pub async fn establish_test_connection() -> Result<AsyncPgConnection> {
    /// --
    use diesel_async::AsyncConnection;
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let conn = AsyncPgConnection::establish(&url).await?;
    Ok(conn)
}

// This test_support module provides support for Diesel-based integration tests without
// relying on the CLI:

/// Inserts a test user into the database using the provided connection.
///
/// # Arguments
/// * `conn` - A mutable reference to an asynchronous Postgres connection.
/// * `new_user` - A reference to a `NewUser` struct with the test user's data.
///
/// # Returns
/// Returns the inserted `User` on success, or an error if the insert fails.
pub async fn insert_test_user(conn: &mut AsyncPgConnection, new_user: &NewUser) -> Result<User> {
    use diesel::insert_into;

    let username = new_user.username.clone();

    let user = AsyncRunQueryDsl::get_result(insert_into(users::table).values(new_user), conn)
        .await
        .context(format!("Failed to insert test user:{username}"))?;

    Ok(user)
}

/// Finds the role ID by code and assigns it to the user in the `user_roles` table.
pub async fn assign_role(
    conn: &mut AsyncPgConnection,
    user_id_value: i32,
    role_code_str: &str,
) -> Result<()> {
    // ---
    use crate::schema::roles::dsl::{code, id as role_id_column, roles};
    use crate::schema::user_roles::dsl::{
        role_id as role_id_col, user_id as user_id_col, user_roles,
    };
    use diesel::insert_into;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl as AsyncRunQueryDsl;

    // Step 1: Look up role ID from the roles table
    let role_id_value = {
        let query = roles.filter(code.eq(role_code_str)).select(role_id_column);

        AsyncRunQueryDsl::first::<i32>(query, conn)
            .await
            .context("Failed to find role by code")?
    };

    // Step 2: Insert into user_roles with resolved IDs
    let insert = insert_into(user_roles)
        .values((user_id_col.eq(user_id_value), role_id_col.eq(role_id_value)));

    AsyncRunQueryDsl::execute(insert, conn)
        .await
        .context("Failed to insert into user_roles")?;

    Ok(())
}
