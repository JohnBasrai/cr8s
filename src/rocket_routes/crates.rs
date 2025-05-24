// rocket_routes/crates.rs
use super::GuardedAppUser;
use crate::domain::{
    //
    CrateTableTraitPtr,
    NewCrate,
};
use crate::rocket_routes::server_error;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::State;

#[rocket::get("/crates")]
pub async fn get_crates(
    repo: &State<crate::domain::CrateTableTraitPtr>,
    _user: GuardedAppUser,
) -> Result<Value, Custom<Value>> {
    // ---
    let crates = repo
        .inner()
        .find_multiple(100)
        .await
        .map_err(server_error)?;

    Ok(json!(crates))
}

#[rocket::post("/crates", format = "json", data = "<new_crate>")]
pub async fn create_crate(
    repo: &State<CrateTableTraitPtr>,
    _user: GuardedAppUser,
    new_crate: Json<NewCrate>,
) -> Result<Value, Custom<Value>> {
    // ---
    let result = repo
        .create(new_crate.into_inner())
        .await
        .map_err(server_error)?;

    Ok(json!(result))
}

#[rocket::get("/crates/<id>")]
pub async fn view_crate(
    repo: &State<CrateTableTraitPtr>,
    _user: GuardedAppUser,
    id: i32,
) -> Result<Value, Custom<Value>> {
    repo.find(id).await.map(|c| json!(c)).map_err(server_error)
}

#[rocket::put("/crates/<id>", format = "json", data = "<a_crate>")]
pub async fn update_crate(
    repo: &State<CrateTableTraitPtr>,
    _user: GuardedAppUser,
    id: i32,
    a_crate: Json<NewCrate>,
) -> Result<Value, Custom<Value>> {
    // --
    let updated = repo
        .update(id, a_crate.into_inner())
        .await
        .map_err(server_error)?;

    Ok(json!(updated))
}

#[rocket::delete("/crates/<id>")]
pub async fn delete_crate(
    repo: &State<CrateTableTraitPtr>,
    _user: GuardedAppUser,
    id: i32,
) -> Result<Value, Custom<Value>> {
    repo.delete(id).await.map_err(server_error)?;

    Ok(json!({ "deleted": true }))
}

#[cfg(test)]
mod tests {
    // ---
    use super::*;
    use crate::domain::{
        //
        AppUser as DomainAppUser,
        Crate as CrateModel,
        CrateTableTrait,
        NewCrate,
    };
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    pub struct MockCrateRepo {
        crates: HashMap<i32, CrateModel>,
    }

    impl MockCrateRepo {
        pub fn new() -> Self {
            Self {
                crates: HashMap::new(),
            }
        }

        pub fn with_crate(mut self, c: CrateModel) -> Self {
            self.crates.insert(c.id, c);
            self
        }
    }

    #[async_trait]
    impl CrateTableTrait for MockCrateRepo {
        async fn find_multiple(&self, _limit: i64) -> Result<Vec<CrateModel>> {
            Ok(self.crates.values().cloned().collect())
        }

        async fn create(&self, new: NewCrate) -> Result<CrateModel> {
            Ok(CrateModel {
                id: 999,
                author_id: new.author_id,
                code: new.code,
                name: new.name,
                version: new.version,
                description: new.description,
                created_at: Utc::now().naive_utc(),
            })
        }

        async fn update(&self, _id: i32, updated: CrateModel) -> Result<CrateModel> {
            Ok(updated)
        }

        async fn delete(&self, _id: i32) -> Result<()> {
            Ok(())
        }

        // Stub unused methods to satisfy trait (if required)
        async fn find(&self, _id: i32) -> Result<CrateModel> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_crates_route_returns_expected_json() -> Result<()> {
        // ---
        let mock_repo = Arc::new(MockCrateRepo::new().with_crate());
        let user = DomainAppUser {
            id: 1,
            username: "test".into(),
        };

        let result = get_crates(mock_repo.into(), user).await?;
        assert_eq!(result["0"]["name"], "test_crate");
        Ok(())
    }

    // ---

    #[tokio::test]
    async fn test_create_crate_success() -> Result<()> {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let user = DomainAppUser {
            id: 42,
            username: "alice".into(),
        };
        let new_crate = Json(NewCrate {
            author_id: 42,
            code: "abc".into(),
            name: "test_create".into(),
            version: "1.0.0".into(),
            description: Some("desc".into()),
        });

        let result = create_crate(repo.into(), user, new_crate).await?;
        assert_eq!(result["name"], "test_create");
        Ok(())
    }

    // --

    #[tokio::test]
    async fn test_view_crate_success() -> Result<()> {
        let test_crate = CrateModel {
            id: 10,
            author_id: 1,
            code: "crate10".into(),
            name: "Test Crate".into(),
            version: "1.0.0".into(),
            description: Some("A crate for testing".into()),
            created_at: Utc::now().naive_utc(),
        };

        let repo = Arc::new(MockCrateRepo::new().with_crate(test_crate.clone()));
        let user = DomainAppUser {
            id: 1,
            username: "test".into(),
        };

        let result = view_crate(repo.into(), user, 10).await?;
        assert_eq!(result["name"], "Test Crate");
        Ok(())
    }

    // --

    #[tokio::test]
    async fn test_update_crate_success() -> Result<()> {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let user = DomainAppUser {
            id: 1,
            username: "bob".into(),
        };
        let updated = Json(NewCrate {
            author_id: 1,
            code: "upd".into(),
            name: "updated_crate".into(),
            version: "2.0.0".into(),
            description: None,
        });

        let result = update_crate(repo.into(), user, 123, updated).await?;
        assert_eq!(result["name"], "updated_crate");
        Ok(())
    }

    // ---

    #[tokio::test]
    async fn test_delete_crate_success() -> Result<()> {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let user = DomainAppUser {
            id: 1,
            username: "admin".into(),
        };

        let result = delete_crate(repo.into(), user, 555).await?;
        assert_eq!(result["deleted"], true);
        Ok(())
    }
    #[tokio::test]
    async fn test_update_crate_success() -> Result<()> {
        let repo = Arc::new(MockCrateRepo::new());
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 42,
            username: "updater".into(),
        });

        let updated = Json(NewCrate {
            author_id: 42,
            code: "updated".into(),
            name: "Updated Crate".into(),
            version: "2.0.0".into(),
            description: Some("Updated description".into()),
        });

        let result = update_crate(repo.into(), user, 123, updated).await?;
        assert_eq!(result["name"], "Updated Crate");
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_crate_success() -> Result<()> {
        let repo = Arc::new(MockCrateRepo::new());
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "deleter".into(),
        });

        let result = delete_crate(repo.into(), user, 999).await?;
        assert_eq!(result["deleted"], true);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_crate_failure() {
        // TODO: Simulate backend failure in create()
        todo!("Implement create_crate() failure path");
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_crate_failure_invalid_id() {
        // TODO: Simulate update failure on unknown ID
        todo!("Implement update_crate() failure path for bad ID");
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_crate_failure_known_bad_id() {
        // TODO: Simulate delete failure for flagged ID
        todo!("Implement delete_crate() failure path");
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_crate_validation_failure() {
        // TODO: Simulate invalid or missing fields in request
        todo!("Implement input validation failure for create_crate()");
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_crate_not_found() {
        // TODO: simulate missing ID in mock and assert Custom(Status::NotFound)
        todo!("Implement error case: crate not found");
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_crate_db_error() {
        // TODO: simulate unexpected backend failure (e.g. connection error)
        todo!("Implement error case: database error");
    }
}
