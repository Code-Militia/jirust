use log::info;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tokio::spawn;

pub type SurrealAny = Surreal<Any>;

use self::projects::Project;
use self::tickets::TicketData;
use self::{
    auth::{jira_authentication, JiraClient},
    projects::JiraProjects,
    tickets::JiraTickets,
};

pub mod auth;
pub mod projects;
pub mod tickets;

pub struct Jira {
    pub client: JiraClient,
    pub db: SurrealAny,
    pub projects: JiraProjects,
    pub tickets: JiraTickets,
}

impl Jira {
    pub async fn new() -> anyhow::Result<Jira> {
        let auth = jira_authentication();
        let db = connect("mem://").await?;
        db.use_ns("noc").use_db("database").await?;
        let projects: JiraProjects = JiraProjects::new().await?;
        let tickets: JiraTickets = JiraTickets::new().await?;

        Ok(Self {
            client: auth,
            db,
            projects,
            tickets,
        })
    }

    pub async fn get_jira_projects(&mut self, get_next_page: bool) -> anyhow::Result<&Vec<Project>> {
        // TODO: Need a way to get previous page
        // TODO: This always gets it from the API.  This needs to try to get from DB first
        match &self.projects.next_page {
            None => {},
            Some(url) if get_next_page && !&self.projects.is_last => {
                let resp = self.projects.get_projects_from_jira_api(&self.client, url.to_string()).await?;
                self.projects = serde_json::from_str(resp.as_str()).expect("projects deserialized");
                return Ok(&self.projects.values)
            },
            &Some(_) => {
                info!("we're in this block");
                todo!("need to handle cases for wildcards")
            }
        }

        let projects: Vec<Project> = self.db.select("project").await?;

        let jira_url = self.client.get_domain();
        let jira_api_version = self.client.get_api_version();
        if projects.is_empty() {
            let url = format!("{}/rest/api/{}/project/search?maxResults=1&startAt=0", jira_url, jira_api_version);
            let resp = self.projects.get_projects_from_jira_api(&self.client, url).await?;
            self.projects = serde_json::from_str(resp.as_str()).expect("projects deserialized");
            for project in &self.projects.values {
                let db = self.db.clone();
                let prj = project.clone();
                spawn(async move {
                    let _projects_insert: Project = db
                        .create("project")
                        .content(prj)
                        .await.expect("projects inserted into db");
                });
            }

            return Ok(&self.projects.values)
        }
        Ok(&self.projects.values)
    }

    pub async fn get_jira_tickets(
        &self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>> {
        let sql = r#"
            SELECT * FROM tickets WHERE fields.project.key = $project_key
            "#;
        let mut query = self.db
            .query(sql)
            .bind(("project_key", format!("{}", project_key)))
            .await?;
        let tickets: Vec<TicketData> = query.take(0)?;
        if tickets.is_empty() {

            let domain = self.client.get_domain();
            let url = format!(
                "{}/rest/api/3/search?jql=project%20%3D%20{}&expand=renderedFields",
                domain, project_key
            );
            let resp = self.tickets.get_tickets_from_jira_api(&self.client, &url).await?;
            let object: JiraTickets = serde_json::from_str(resp.as_str())
                .expect("unable to convert project resp to slice");
            for ticket in object.issues.iter() {
                let _: TicketData = self.db.create(("tickets", &ticket.key)).content(&ticket).await?;
            }

            return Ok(object.issues);
        }
        Ok(tickets)
    }
}
