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
    pub project_start_at: u32,
    pub project_max_results: u32,
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
            project_start_at: 0,
            project_max_results: 2,
            tickets,
        })
    }

    pub async fn get_jira_projects(&mut self, get_next_page: bool, get_previous_page: bool) -> anyhow::Result<Vec<Project>> {

        // Search for next project in db
        if get_next_page {
            self.project_start_at += self.project_max_results;
            let mut query = self.db.query(format!("SELECT * FROM projects LIMIT {} START {}", self.project_max_results, self.project_start_at)).await.expect("projects selected");
            let projects: Vec<Project> = query.take(0)?;
            if !projects.is_empty() {
                self.projects.values = projects;
                return Ok(self.projects.values.clone());
            }
            self.project_start_at -= self.project_max_results;
        }

        // Search for next project in Jira API
        match &self.projects.next_page {
            None => {},
            Some(next_page_url) if get_next_page => {
                self.project_start_at += self.project_max_results;
                let resp = self.projects.get_projects_from_jira_api(&self.client, next_page_url.to_string()).await?;
                self.projects = serde_json::from_str(resp.as_str()).expect("projects deserialized");
                for project in &self.projects.values {
                    let db = self.db.clone();
                    let prj = project.clone();
                    spawn(async move {
                        let _projects_insert: Project = db
                            .create(("projects", &prj.key))
                            .content(prj)
                            .await.expect("projects inserted into db");
                    });
                }
                return Ok(self.projects.values.clone())
            },
            Some(_) => {
                // info!("value of wildcard {:?}", x);
                // info!("we're in this block");
                // todo!("need to handle cases for wildcards")
            }
        }

        // Previous project request from database
        if get_previous_page && self.project_start_at >= 1 {
            self.project_start_at -= self.project_max_results;
        }

        let mut query = self.db.query(format!("SELECT * FROM projects LIMIT {} START {}", self.project_max_results, self.project_start_at)).await.expect("projects selected");
        let projects: Vec<Project> = query.take(0)?;

        // Get initiall projects request
        if projects.is_empty() {
            let jira_url = self.client.get_domain();
            let jira_api_version = self.client.get_api_version();
            let url = format!("{}/rest/api/{}/project/search?maxResults={}&startAt=0", jira_url, jira_api_version, self.project_max_results);
            let resp = self.projects.get_projects_from_jira_api(&self.client, url).await?;
            self.projects = serde_json::from_str(resp.as_str()).expect("projects deserialized");
            for project in &self.projects.values {
                let db = self.db.clone();
                let prj = project.clone();
                spawn(async move {
                    let _projects_insert: Project = db
                        .create(("projects", &prj.key))
                        .content(prj)
                        .await.expect("projects inserted into db");
                });
            }

            return Ok(self.projects.values.clone())
        }
        self.projects.values = projects;
        Ok(self.projects.values.clone())
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
                let db = self.db.clone();
                let tkt= ticket.clone();
                spawn(async move {
                    let _ticket_insert: TicketData = db
                        .create(("tickets", &tkt.key))
                        .content(tkt)
                        .await.expect("Ticket inserted to db");
                    // TODO: Update projects record to add a link to ticket records created here
                });
            }

            return Ok(object.issues);
        }
        Ok(tickets)
    }
}
