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
        CrateSummary,
        CrateTableTrait,
        NewCrate,
    };
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use chrono::Utc;
    use rocket::State;
    use std::collections::HashMap;
    use std::sync::Arc;

    pub struct MockCrateRepo {
        crates: HashMap<i32, CrateModel>,
    }

    impl MockCrateRepo {
        // ---
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
        // ---
        async fn find_multiple(&self, _limit: i64) -> Result<Vec<CrateModel>> {
            Ok(self.crates.values().cloned().collect())
        }

        async fn create(&self, new: NewCrate) -> Result<CrateModel> {
            // ---
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

        async fn update(&self, _id: i32, updated: NewCrate) -> Result<CrateModel> {
            // ---
            Ok(CrateModel {
                id: _id,
                author_id: updated.author_id,
                code: updated.code,
                name: updated.name,
                version: updated.version,
                description: updated.description,
                created_at: Utc::now().naive_utc(),
            })
        }

        async fn delete(&self, _id: i32) -> Result<()> {
            // ---
            Ok(())
        }

        async fn find(&self, id: i32) -> Result<CrateModel> {
            // ---
            self.crates
                .get(&id)
                .cloned()
                .ok_or_else(|| anyhow!("Crate not found"))
        }

        async fn find_since(&self, _hours_since: i32) -> Result<Vec<CrateSummary>> {
            // ---
            let summaries = self
                .crates
                .values()
                .map(|c| CrateSummary {
                    name: c.name.clone(),
                    version: c.version.clone(),
                })
                .collect();
            Ok(summaries)
        }
    }

    #[tokio::test]
    async fn test_get_crates_route_returns_expected_json() {
        // ---
        let test_crate = CrateModel {
            id: 1,
            author_id: 1,
            code: "test_code".into(),
            name: "test_crate".into(),
            version: "1.0.0".into(),
            description: Some("Test description".into()),
            created_at: Utc::now().naive_utc(),
        };

        let mock_repo = Arc::new(MockCrateRepo::new().with_crate(test_crate));
        let binding = mock_repo as Arc<dyn CrateTableTrait>;
        let repo_state = State::from(&binding);
        let user = GuardedAppUser(DomainAppUser {
            id: 1,
            username: "test".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = get_crates(repo_state, user).await;
        match result {
            Ok(value) => {
                assert_eq!(value[0]["name"], "test_crate");
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_create_crate_success() {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let binding = repo as Arc<dyn CrateTableTrait>;
        let repo_state = State::from(&binding);
        let user = GuardedAppUser(DomainAppUser {
            id: 42,
            username: "alice".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });
        let new_crate = Json(NewCrate {
            author_id: 42,
            code: "abc".into(),
            name: "test_create".into(),
            version: "1.0.0".into(),
            description: Some("desc".into()),
        });

        let result = create_crate(repo_state, user, new_crate).await;
        match result {
            Ok(value) => {
                assert_eq!(value["name"], "test_create");
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_view_crate_success() {
        // ---
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
        let binding = repo as Arc<dyn CrateTableTrait>;
        let repo_state = State::from(&binding);
        let user = GuardedAppUser(DomainAppUser {
            id: 1,
            username: "test".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = view_crate(repo_state, user, 10).await;
        match result {
            Ok(value) => {
                assert_eq!(value["name"], "Test Crate");
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_crate_success() {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let binding = repo as Arc<dyn CrateTableTrait>;
        let repo_state = State::from(&binding);
        let user = GuardedAppUser(DomainAppUser {
            id: 1,
            username: "bob".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });
        let updated = Json(NewCrate {
            author_id: 1,
            code: "upd".into(),
            name: "updated_crate".into(),
            version: "2.0.0".into(),
            description: None,
        });

        let result = update_crate(repo_state, user, 123, updated).await;
        match result {
            Ok(value) => {
                assert_eq!(value["name"], "updated_crate");
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_crate_success() {
        // ---
        let repo = Arc::new(MockCrateRepo::new());
        let binding = repo as Arc<dyn CrateTableTrait>;
        let repo_state = State::from(&binding);
        let user = GuardedAppUser(DomainAppUser {
            id: 1,
            username: "admin".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = delete_crate(repo_state, user, 555).await;
        match result {
            Ok(value) => {
                assert_eq!(value["deleted"], true);
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_crate_failure() {
        // ---
        todo!("Implement create_crate() failure path");
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_crate_failure_invalid_id() {
        // ---
        todo!("Implement update_crate() failure path for bad ID");
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_crate_failure_known_bad_id() {
        // ---
        todo!("Implement delete_crate() failure path");
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_crate_validation_failure() {
        // ---
        todo!("Implement input validation failure for create_crate()");
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_crate_not_found() {
        // ---
        todo!("Implement error case: crate not found");
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_crate_db_error() {
        // ---
        todo!("Implement error case: database error");
    }
}
