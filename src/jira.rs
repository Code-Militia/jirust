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
    pub tickets_start_at: u32,
    pub tickets_max_results: u32,
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
            tickets_start_at: 0,
            tickets_max_results: 2,
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
        &mut self,
        project_key: &str,
        get_next_page: bool,
        get_previous_page: bool
    ) -> anyhow::Result<Vec<TicketData>> {
        // Search for next ticket page in db
        if get_next_page {
            self.tickets_start_at += self.tickets_max_results;
            let mut query = self.db.query(format!("SELECT * FROM tickets LIMIT {} START {}", self.tickets_max_results, self.tickets_start_at)).await?;
            let tickets: Vec<TicketData> = query.take(0)?;
            if !tickets.is_empty() {
                self.tickets.issues = tickets;
                return Ok(self.tickets.issues.clone());
            }
            self.tickets_start_at -= self.tickets_max_results;

            if (self.tickets_start_at + self.tickets_max_results) < self.tickets.total {
                let jira_url = self.client.get_domain();
                let next_page_url = format!(
                    "{}/rest/api/3/search?maxResults={}&startAt={}&jql=project%20%3D%20{}&expand=renderedFields",
                     jira_url, self.tickets_max_results, (self.tickets_start_at + self.tickets_max_results), project_key
                );
                let resp = self.tickets.get_tickets_from_jira_api(&self.client, next_page_url.as_str()).await?;
                self.tickets = serde_json::from_str(resp.as_str()).expect("tickets deserialized");
                info!("tickets in next page -- {:?}", self.tickets.issues);
                for ticket in &self.tickets.issues {
                    let db = self.db.clone();
                    let tkt = ticket.clone();
                    spawn(async move {
                        let _tickets_insert: TicketData = db
                            .create(("tickets", &tkt.key))
                            .content(tkt)
                            .await.expect("tickets inserted into db");
                    });
                }
                return Ok(self.tickets.issues.clone())

            }
        }

        // Previous project request from database
        if get_previous_page && self.tickets_start_at >= 1 {
            self.tickets_start_at -= self.tickets_max_results;
        }

        let mut query = self.db.query(format!("SELECT * from tickets WHERE fields.project.key = {} LIMIT {} START {}", project_key, self.tickets_max_results, self.tickets_start_at)).await?;
        let tickets: Vec<TicketData> = query.take(0)?;
        info!("tickets resp-- {:?}", query);
        info!("tickets -- {:?}", tickets);
        if tickets.is_empty() {

            let jira_url = self.client.get_domain();
            let url = format!(
                "{}/rest/api/3/search?maxResults={}&startAt=0&jql=project%20%3D%20{}&expand=renderedFields",
                jira_url, self.tickets_max_results, project_key
            );
            let resp = self.tickets.get_tickets_from_jira_api(&self.client, &url).await?;
            self.tickets = serde_json::from_str(resp.as_str())
                .expect("unable to convert project resp to slice");
            for ticket in &self.tickets.issues {
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

            return Ok(self.tickets.issues.clone());
        }
        self.tickets.issues = tickets;
        Ok(self.tickets.issues.clone())
    }
}
