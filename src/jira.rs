use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub type SurrealAny = Surreal<Any>;

use self::{
    auth::{jira_authentication, JiraAuth},
    tickets::JiraTickets,
    projects::JiraProjects,
};

pub mod auth;
pub mod tickets;
pub mod projects;

pub struct Jira {
    pub auth: JiraAuth,
    pub db: Surreal<Any>,
    pub projects: JiraProjects,
    pub tickets: JiraTickets,
}

impl Jira {
    pub async fn new() -> anyhow::Result<Jira> {
        let auth = jira_authentication();
        let db = connect("mem://").await?;
        db.use_ns("noc").use_db("database").await?;
        let projects: JiraProjects = JiraProjects::new(&auth, &db).await?;
        let tickets: JiraTickets = JiraTickets::new().await?;

        Ok(Self {
            auth,
            db,
            projects,
            tickets,
        })
    }
}
