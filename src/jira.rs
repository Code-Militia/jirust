use surrealdb::{Datastore, Session};

use self::auth::{JiraAuth, jira_authentication};

pub mod auth;
pub mod issue;
pub mod jira_db;
pub mod projects;

pub struct Jira {
    pub auth: JiraAuth,
    pub db: (Datastore, Session),
}

pub type DB = (Datastore, Session);

impl Jira {
    pub async fn new() -> anyhow::Result<Jira> {
        let auth = jira_authentication();
        let db: DB = (
            Datastore::new("memory").await?,
            Session::for_db("jira", "jira"),
        );

        Ok(Self {
            auth,
            db,
        })
    }
}
