use log::info;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub type SurrealAny = Surreal<Any>;

use self::tickets::TicketData;
use self::{
    auth::{jira_authentication, JiraAuth},
    projects::JiraProjects,
    tickets::JiraTickets,
};

pub mod auth;
pub mod projects;
pub mod tickets;

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
