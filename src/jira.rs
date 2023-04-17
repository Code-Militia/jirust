use serde::Deserialize;
use serde::Serialize;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBTicketData {
    pub id: String,
    pub key: String,
    pub tickets: Vec<TicketData>,
}

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
    pub async fn new() -> anyhow::Result<Jira, anyhow::Error> {
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
            project_max_results: 50,
            tickets_start_at: 0,
            tickets_max_results: 50,
            tickets,
        })
    }

    pub async fn get_next_project_page(&mut self) -> anyhow::Result<&Vec<Project>, anyhow::Error> {
        self.project_start_at += self.project_max_results;
        let mut query = self
            .db
            .query(format!(
                "SELECT * FROM projects LIMIT {} START {}",
                self.project_max_results, self.project_start_at
            ))
            .await
            .expect("projects selected");
        let projects: Vec<Project> = query.take(0)?;
        if !projects.is_empty() {
            self.projects.values = projects;
            return Ok(&self.projects.values);
        }

        self.project_start_at -= self.project_max_results;
        self.projects = self.projects.get_projects_next_page(&self.client).await?;

        for project in &self.projects.values {
            let db = self.db.clone();
            let prj = project.clone();
            spawn(async move {
                let _projects_insert: Project = db
                    .create(("projects", &prj.key))
                    .content(prj)
                    .await
                    .expect("projects inserted into db");
            });
        }
        Ok(&self.projects.values)
    }

    pub async fn get_projects_previous_page(
        &mut self,
    ) -> anyhow::Result<Vec<Project>, anyhow::Error> {
        if self.project_start_at >= 1 {
            self.project_start_at -= self.project_max_results;
        }
        self.get_jira_projects().await
    }

    pub async fn get_jira_projects(&mut self) -> anyhow::Result<Vec<Project>, anyhow::Error> {
        let mut query = self
            .db
            .query(format!(
                "SELECT * FROM projects LIMIT {} START {}",
                self.project_max_results, self.project_start_at
            ))
            .await
            .expect("projects selected");
        let projects: Vec<Project> = query.take(0)?;

        // Get initiall projects request
        if projects.is_empty() {
            let jira_url = self.client.get_domain();
            let jira_api_version = self.client.get_api_version();
            let url = format!(
                "{}/rest/api/{}/project/search?maxResults={}&startAt=0",
                jira_url, jira_api_version, self.project_max_results
            );
            let resp = self
                .projects
                .get_projects_from_jira_api(&self.client, url)
                .await?;
            self.projects = serde_json::from_str(resp.as_str()).expect("projects deserialized");
            for project in &self.projects.values {
                let db = self.db.clone();
                let prj = project.clone();
                spawn(async move {
                    let _projects_insert: Project = db
                        .create(("projects", &prj.key))
                        .content(prj)
                        .await
                        .expect("projects inserted into db");
                });
            }

            return Ok(self.projects.values.clone());
        }
        self.projects.values = projects;
        Ok(self.projects.values.clone())
    }

    pub async fn get_and_record_tickets(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        let jira_url = self.client.get_domain();
        let url = format!(
            "{}/rest/api/3/search?maxResults={}&startAt=0&jql=project%20%3D%20{}&expand=renderedFields",
            jira_url, self.tickets_max_results, project_key
        );
        let resp = self
            .tickets
            .get_tickets_from_jira_api(&self.client, &url)
            .await?;
        self.tickets = serde_json::from_str(resp.as_str()).expect("tickets deserialized");
        for ticket in &self.tickets.issues {
            let db = self.db.clone();
            let tkt = ticket.clone();
            spawn(async move {
                let _tickets_insert: TicketData = db
                    .update(("tickets", &tkt.key))
                    .content(tkt)
                    .await
                    .expect("tickets inserted into db");
            });
        }
        Ok(self.tickets.issues.clone())
    }

    pub async fn get_next_ticket_page(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        self.tickets_start_at += self.tickets_max_results;
        let sql = format!(
            "SELECT * FROM tickets WHERE fields.project.key = '{}' LIMIT {} START {}",
            project_key, self.tickets_max_results, self.tickets_start_at
        );
        let mut query = self.db.query(sql).await?;
        let tickets: Vec<TicketData> = query.take(0)?;
        if !tickets.is_empty() {
            self.tickets.issues = tickets;
            return Ok(self.tickets.issues.clone());
        }
        self.tickets_start_at -= self.tickets_max_results;

        if (self.tickets_start_at + self.tickets_max_results) < self.tickets.total {
            self.tickets_start_at += self.tickets_max_results;
            let jira_url = self.client.get_domain();
            let next_page_url = format!(
                    "{}/rest/api/3/search?maxResults={}&startAt={}&jql=project%20%3D%20{}&expand=renderedFields",
                    jira_url, self.tickets_max_results, (self.tickets_start_at + self.tickets_max_results), project_key
                );
            return self.get_and_record_tickets(&next_page_url).await;
        }
        Ok(self.tickets.issues.clone())
    }

    pub async fn get_previous_tickets_page(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        self.tickets_start_at = self.tickets_start_at.saturating_sub(self.tickets_max_results);
        // self.tickets_start_at = self
        //     .tickets_start_at
        //     .checked_sub(self.tickets_max_results)
        //     .unwrap_or(0);
        self.get_jira_tickets(project_key).await
    }

    pub async fn get_jira_tickets(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        let sql = format!(
            "SELECT * FROM tickets WHERE fields.project.key = '{}' LIMIT {} START {}",
            project_key, self.tickets_max_results, self.tickets_start_at
        );
        let mut query = self.db.query(sql).await.ok().unwrap();
        let tickets: Vec<TicketData> = query.take(0)?;
        if tickets.is_empty() {
            self.get_and_record_tickets(project_key).await?;
            return Ok(self.tickets.issues.clone());
        }
        Ok(self.tickets.issues.clone())
    }
}
