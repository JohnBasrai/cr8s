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

    repo.inner()
        .update(id, author.into_inner())
        .await
        .map(|author| json!(author))
        .map_err(server_error)
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
    use crate::domain::{
        //
        Author,
        AuthorTableTrait,
        AuthorTableTraitPtr,
        NewAuthor,
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use rocket::serde::json::json;
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
            })
        }

        async fn update(&self, _id: i32, updated: Author) -> Result<Author> {
            Ok(updated)
        }

        async fn delete(&self, _id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_authors_returns_list() -> Result<()> {
        let author = Author {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
            created_at: Utc::now().naive_utc(),
        };

        let repo = Arc::new(MockAuthorRepo::new().with_author(author.clone()));
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
        });

        let result = get_rustaceans(repo.into(), user).await?;
        assert_eq!(result[0]["name"], "Alice");
        Ok(())
    }

    #[tokio::test]
    async fn test_view_author_success() -> Result<()> {
        let author = Author {
            id: 5,
            name: "Bob".into(),
            email: "bob@example.com".into(),
            created_at: Utc::now().naive_utc(),
        };

        let repo = Arc::new(MockAuthorRepo::new().with_author(author.clone()));
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
        });

        let result = view_rustacean(repo.into(), 5, user).await?;
        assert_eq!(result["email"], "bob@example.com");
        Ok(())
    }

    #[tokio::test]
    async fn test_create_author_success() -> Result<()> {
        let repo = Arc::new(MockAuthorRepo::new());
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 123,
            username: "tester".into(),
        });

        let new_author = Json(NewAuthor {
            name: "Charlie".into(),
            email: "charlie@example.com".into(),
        });

        let result = create_rustacean(repo.into(), user, new_author).await?;
        assert_eq!(result["name"], "Charlie");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_view_author_not_found() {
        todo!("Handle error case where view_rustacean is called with unknown ID");
    }

    #[tokio::test]
    async fn test_update_author_success() -> Result<()> {
        // ---
        let existing = Author {
            id: 10,
            name: "Old Name".into(),
            email: "old@example.com".into(),
            created_at: Utc::now().naive_utc(),
        };

        let repo = Arc::new(MockAuthorRepo::new().with_author(existing));
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "admin".into(),
        });

        let updated = Json(NewAuthor {
            name: "Updated Name".into(),
            email: "updated@example.com".into(),
        });

        let result = update_rustacean(repo.into(), user, 10, updated).await?;
        assert_eq!(result["name"], "Updated Name");
        assert_eq!(result["email"], "updated@example.com");
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_author_success() -> Result<()> {
        // ---
        let author = Author {
            id: 7,
            name: "ToDelete".into(),
            email: "delete@example.com".into(),
            created_at: Utc::now().naive_utc(),
        };

        let repo = Arc::new(MockAuthorRepo::new().with_author(author));
        let user = GuardedAppUser(crate::domain::AppUser {
            id: 1,
            username: "admin".into(),
        });

        let result = delete_rustacean(repo.into(), user, 7).await?;
        assert_eq!(result["deleted"], true);
        Ok(())
    }
}
