use surrealdb::{Datastore, Session};
use super::auth::{JiraAuth, jira_authentication};
// use super::projects::JiraProjects;
use tuirealm::Props;
use std::error::Error;

struct Projects {
    props: Props,
    states: ProjectState
}

struct ProjectState {
    list: Vec<String>
}

impl ProjectState {
    async fn get_projct_list(&self) -> Result<(), Error> {
        Ok(())
    }
}
