//! 🚧 This module maintains the legacy `/rustaceans` REST API.  Internally
//! backed by the renamed `Author` domain model and traits.  See tracking issue
//! (#23) in cr8s and (#16) cr8s-fe to rename this endpoint.

use super::{EditorUser, GuardedAppUser};
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, State};

use super::server_error;
use crate::domain::{
    //
    Author,
    AuthorTableTraitPtr,
    NewAuthor,
};

#[get("/rustaceans")]
pub async fn get_rustaceans(
    repo: &State<AuthorTableTraitPtr>,
    _user: GuardedAppUser,
) -> Result<Value, Custom<Value>> {
    // ---

    repo.inner()
        .find_multiple(100)
        .await
        .map(|authors| json!(authors))
        .map_err(server_error)
}

#[get("/rustaceans/<id>")]
pub async fn view_rustacean(
    repo: &State<AuthorTableTraitPtr>,
    id: i32,
    _user: GuardedAppUser,
) -> Result<Value, Custom<Value>> {
    // ---

    repo.inner()
        .find(id)
        .await
        .map(|author| json!(author))
        .map_err(server_error)
}

#[post("/rustaceans", format = "json", data = "<new_author>")]
pub async fn create_rustacean(
    repo: &State<AuthorTableTraitPtr>,
    new_author: Json<NewAuthor>,
    _user: EditorUser,
) -> Result<Custom<Value>, Custom<Value>> {
    // ---

    repo.inner()
        .create(new_author.into_inner())
        .await
        .map(|author| Custom(Status::Created, json!(author)))
        .map_err(server_error)
}

#[put("/rustaceans/<id>", format = "json", data = "<author>")]
pub async fn update_rustacean(
    repo: &State<AuthorTableTraitPtr>,
    id: i32,
    author: Json<Author>,
    _user: EditorUser,
) -> Result<Value, Custom<Value>> {
    // ---

    let update_conflict =
        "Update conflict: row was modified by another user. Please refresh and try again.";

    repo.inner()
        .update(id, author.row_version, author.into_inner())
        .await
        .map(|author| json!(author))
        .map_err(|e| {
            if e.to_string().contains("no rows") {
                Custom(Status::Conflict, json!({"error": update_conflict}))
            } else {
                server_error(e)
            }
        })
}

#[delete("/rustaceans/<id>")]
pub async fn delete_rustacean(
    repo: &State<AuthorTableTraitPtr>,
    id: i32,
    _user: EditorUser,
) -> Result<NoContent, Custom<Value>> {
    // ---

    repo.inner()
        .delete(id)
        .await
        .map(|_| NoContent)
        .map_err(server_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Author, AuthorTableTrait, NewAuthor};
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use chrono::Utc;
    use rocket::State;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockAuthorRepo {
        authors: HashMap<i32, Author>,
    }

    impl MockAuthorRepo {
        fn new() -> Self {
            Self {
                authors: HashMap::new(),
            }
        }

        fn with_author(mut self, author: Author) -> Self {
            self.authors.insert(author.id, author);
            self
        }
    }

    #[async_trait]
    impl AuthorTableTrait for MockAuthorRepo {
        async fn find(&self, id: i32) -> Result<Author> {
            self.authors
                .get(&id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("not found"))
        }

        async fn find_multiple(&self, _limit: i64) -> Result<Vec<Author>> {
            Ok(self.authors.values().cloned().collect())
        }

        async fn create(&self, new: NewAuthor) -> Result<Author> {
            Ok(Author {
                id: 42,
                name: new.name,
                email: new.email,
                created_at: Utc::now().naive_utc(),
                row_version: 0,
            })
        }

        async fn update(&self, _id: i32, _row_version: i32, updated: Author) -> Result<Author> {
            Ok(updated)
        }

        async fn delete(&self, _id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_authors_returns_list() {
        let author = Author {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 0,
        };

        let repo: Arc<dyn AuthorTableTrait + Send + Sync> =
            Arc::new(MockAuthorRepo::new().with_author(author.clone()));
        let repo_state = State::from(&repo);
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = get_rustaceans(repo_state, user).await;
        match result {
            Ok(value) => {
                assert_eq!(value[0]["name"], "Alice");
            }
            Err(e) => panic!("Expected success but got error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_view_author_success() {
        let author = Author {
            id: 5,
            name: "Bob".into(),
            email: "bob@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 0,
        };

        let repo: Arc<dyn AuthorTableTrait + Send + Sync> =
            Arc::new(MockAuthorRepo::new().with_author(author.clone()));
        let repo_state = State::from(&repo);
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        });

        let result = view_rustacean(repo_state, 5, user).await;
        match result {
            Ok(value) => {
                assert_eq!(value["email"], "bob@example.com");
            }
            Err(e) => panic!("Expected success but got error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_create_author_success() {
        let repo: Arc<dyn AuthorTableTrait + Send + Sync> = Arc::new(MockAuthorRepo::new());
        let repo_state = State::from(&repo);
        let user = EditorUser(GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        }));

        let new_author = Json(NewAuthor {
            name: "Charlie".into(),
            email: "charlie@example.com".into(),
        });

        let result = create_rustacean(repo_state, new_author, user).await;
        match result {
            Ok(custom_response) => {
                let value = custom_response.1; // Extract the JSON value from Custom<Value>
                assert_eq!(value["name"], "Charlie");
            }
            Err(e) => panic!("Expected success but got error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_update_author_success() {
        let existing = Author {
            id: 10,
            name: "Old Name".into(),
            email: "old@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 0,
        };

        let repo: Arc<dyn AuthorTableTrait + Send + Sync> =
            Arc::new(MockAuthorRepo::new().with_author(existing));
        let repo_state = State::from(&repo);
        let user = EditorUser(GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "admin".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        }));

        let updated = Json(Author {
            id: 10,
            name: "Updated Name".into(),
            email: "updated@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 0,
        });

        let result = update_rustacean(repo_state, 10, updated, user).await;

        match result {
            Ok(value) => {
                assert_eq!(value["name"], "Updated Name");
                assert_eq!(value["email"], "updated@example.com");
            }
            Err(e) => panic!("Expected success but got error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_author_success() {
        let author = Author {
            id: 7,
            name: "ToDelete".into(),
            email: "delete@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 0,
        };

        let repo: Arc<dyn AuthorTableTrait + Send + Sync> =
            Arc::new(MockAuthorRepo::new().with_author(author));
        let repo_state = State::from(&repo);
        let user = EditorUser(GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "admin".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        }));

        let result = delete_rustacean(repo_state, 7, user).await;
        match result {
            Ok(_no_content) => {
                // NoContent doesn't have indexable content, so we just check that it succeeded
                // NoContent == success
            }
            Err(e) => panic!("Expected success but got error: {e:?}"),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_author_not_found() {
        todo!("Handle error case where view_rustacean is called with unknown ID");
    }

    #[rocket::async_test]
    async fn test_update_author_version_conflict() {
        // Simulate the database having version 2, but client sends version 1
        struct ConflictRepo;

        #[async_trait]
        impl AuthorTableTrait for ConflictRepo {
            async fn find(&self, _id: i32) -> Result<Author> {
                Ok(Author {
                    id: 1,
                    name: "Old".into(),
                    email: "old@example.com".into(),
                    created_at: Utc::now().naive_utc(),
                    row_version: 2, // Database has version 2
                })
            }

            async fn update(
                &self,
                _id: i32,
                _current_version: i32,
                _author: Author,
            ) -> Result<Author> {
                // Simulate version mismatch - no rows updated
                Err(anyhow!(
                    "no rows returned by a query that expected to return at least one row"
                ))
            }

            // ... other required trait methods with default implementations
            async fn create(&self, _new: NewAuthor) -> Result<Author> {
                unimplemented!()
            }
            async fn find_multiple(&self, _limit: i64) -> Result<Vec<Author>> {
                unimplemented!()
            }
            async fn delete(&self, _id: i32) -> Result<()> {
                unimplemented!()
            }
        }

        let repo: Arc<dyn AuthorTableTrait + Send + Sync> = Arc::new(ConflictRepo);
        let repo_state = State::from(&repo);
        // wrap GuardedAppUser
        let user = EditorUser(GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "tester".into(),
            password: "password".into(),
            created_at: Utc::now().naive_utc(),
        }));

        let updated = Json(Author {
            id: 1,
            name: "New Name".into(),
            email: "new@example.com".into(),
            created_at: Utc::now().naive_utc(),
            row_version: 1, // Client thinks version is 1
        });

        let result = update_rustacean(repo_state, 1, updated, user).await;

        // Should return 409 Conflict
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.0, Status::Conflict);

        let error_json = err.1;
        assert!(error_json["error"]
            .as_str()
            .unwrap()
            .contains("Update conflict"));
    }
}
