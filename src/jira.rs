use log::debug;
use serde::Deserialize;
use serde::Serialize;
use surrealdb::engine::any::connect;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub type SurrealAny = Surreal<Any>;

use crate::config::JiraConfigProjects;
use crate::config::JiraConfigTickets;

use self::projects::Project;
use self::tickets::TicketData;
use self::{
    auth::{jira_authentication, JiraClient},
    projects::JiraProjectsAPI,
    tickets::JiraTicketsAPI,
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
    pub projects_api: JiraProjectsAPI,
    pub project_start_at: u32,
    pub project_max_results: u32,
    pub tickets_start_at: u32,
    pub tickets_max_results: u32,
    pub tickets_api: JiraTicketsAPI,
    pub user_config_projects: Option<JiraConfigProjects>,
    pub user_config_tickets: Option<JiraConfigTickets>,
}

impl Jira {
    pub async fn new(
        domain: &str,
        api_key: &str,
        api_version: &str,
        current_user_email: &str,
        user_config_cache: &Option<bool>,
        user_config_project: &Option<JiraConfigProjects>,
        user_config_tickets: &Option<JiraConfigTickets>,
    ) -> anyhow::Result<Jira, anyhow::Error> {
        let auth = jira_authentication(domain, api_key, api_version, current_user_email);
        let projects: JiraProjectsAPI = JiraProjectsAPI::new().await?;
        let tickets: JiraTicketsAPI = JiraTicketsAPI::new().await?;
        let db = match user_config_cache {
            Some(_) => connect("file:///tmp/jirust.db").await?,
            None => connect("mem://").await?,
        };
        db.use_ns("noc").use_db("database").await?;

        Ok(Self {
            client: auth,
            db,
            projects_api: projects,
            project_start_at: 0,
            project_max_results: 50,
            tickets_start_at: 0,
            tickets_max_results: 50,
            tickets_api: tickets,
            user_config_projects: user_config_project.clone(),
            user_config_tickets: user_config_tickets.clone(),
        })
    }

    pub async fn clear_projects_table(&mut self) -> anyhow::Result<()> {
        let _delete_projects: Vec<Project> = self.db.delete("projects").await?;
        Ok(())
    }

    pub async fn get_next_project_page(&mut self) -> anyhow::Result<&Vec<Project>, anyhow::Error> {
        self.project_start_at += self.project_max_results;
        let mut query = self
            .db
            .query("SELECT * FROM projects START type::number($start_at)")
            .bind(("start_at", self.project_start_at))
            .await
            .expect("projects selected");
        let projects: Vec<Project> = query.take(0)?;
        if !projects.is_empty() {
            self.projects_api.values = projects;
            return Ok(&self.projects_api.values);
        }

        self.projects_api.values.clear();
        self.project_start_at -= self.project_max_results;
        self.projects_api = self
            .projects_api
            .get_projects_next_page(&self.client)
            .await?;

        for project in &self.projects_api.values {
            let _projects_insert: Project = self
                .db
                .update(("projects", &project.key))
                .content(project)
                .await?
                .expect("projects inserted into db");
        }
        Ok(&self.projects_api.values)
    }

    pub async fn get_projects_previous_page(
        &mut self,
    ) -> anyhow::Result<Vec<Project>, anyhow::Error> {
        if self.project_start_at >= 1 {
            self.project_start_at -= self.project_max_results;
        }
        self.projects_api.values.clear();
        self.get_jira_projects().await
    }

    pub async fn get_jira_projects(&mut self) -> anyhow::Result<Vec<Project>, anyhow::Error> {
        let mut query = self
            .db
            .query(
                "SELECT * FROM projects LIMIT type::number($limit) START type::number($start_at)",
            )
            .bind(("limit", self.project_max_results))
            .bind(("start_at", self.project_start_at))
            .await
            .expect("projects selected");
        let projects: Vec<Project> = query.take(0)?;
        debug!("Projects found on cache {:?}", projects);

        // Get initial projects request
        if projects.is_empty() {
            let jira_url = self.client.get_domain();
            let mut url = format!(
                "{}/project/search?maxResults={}&startAt=0",
                jira_url, self.project_max_results
            );
            if self.user_config_projects.is_some() {
                let projects = self.user_config_projects.as_ref().unwrap();
                url = format!(
                    "{}/project/search?keys={}",
                    jira_url, projects.default_projects
                )
            }
            let resp = self
                .projects_api
                .get_projects_from_jira_api(&self.client, url)
                .await?;
            self.projects_api = serde_json::from_str(resp.as_str()).expect("projects deserialized");

            debug!("Projects found from JIRA {:?}", self.projects_api);
            for project in &self.projects_api.values {
                let _projects_insert: Project = self
                    .db
                    .update(("projects", &project.key))
                    .content(project)
                    .await?
                    .expect("projects inserted into db");
            }

            return Ok(self.projects_api.values.clone());
        }
        self.projects_api.values = projects;
        Ok(self.projects_api.values.clone())
    }

    pub async fn clear_tickets_table(&mut self) -> anyhow::Result<()> {
        let _delete_projects: Vec<TicketData> = self.db.delete("tickets").await?;
        Ok(())
    }

    pub async fn get_and_record_tickets(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        debug!("Retrieve tickets from API project {project_key}");
        let jira_url = self.client.get_domain();
        let url = format!("{}/search", jira_url);
        let max_results = self.tickets_max_results.to_string();
        let start_at = self.tickets_start_at.to_string();
        let mut jql = format!("project = {}", project_key);
        if self.user_config_tickets.is_some() {
            let config_tickets = self.user_config_tickets.clone().unwrap();
            if let Some(current_user_tickets) = config_tickets.current_user_tickets_only {
                if current_user_tickets {
                    jql = format!("{jql} AND assignee = currentUser()")
                }
                debug!("{jql}");
            }
            if config_tickets.current_sprint_tickets_only == Some(true) {
                jql = format!("{jql} AND sprint IN openSprints()")
            }
            debug!("{jql}");

            let mut ticket_status: Vec<String> = vec![];
            if let Some(specified_ticket_status) = config_tickets.show_ticket_status {
                ticket_status = specified_ticket_status
            }
            if !ticket_status.is_empty() {
                for (index, status) in ticket_status.iter().enumerate() {
                    if index == 0 {
                        jql = format!("{jql} AND (Status = \"{status}\"");
                        debug!("{jql}");
                    } else {
                        jql = format!("{jql} OR Status = \"{status}\"");
                        debug!("{jql}");
                    }
                }
                jql += ")";
                debug!("{jql}");
            }
        }
        let params = vec![
            ("maxResults", max_results.as_ref()),
            ("jql", jql.as_ref()),
            ("expand", "renderedFields"),
            ("startAt", start_at.as_ref()),
            ("fields", "assignee,comments,components,creator,description,issuelinks,issuetype,labels,parent,priority,project,reporter,status,summary")
        ];
        debug!("JQL {:?}", params);
        let resp = self
            .tickets_api
            .get_tickets_api(&self.client, params, &url)
            .await?;
        debug!("{resp}");
        self.tickets_api = serde_json::from_str(resp.as_str()).expect("tickets deserialized");
        for ticket in self.tickets_api.issues.clone() {
            let tickets_insert: TicketData = self
                .db
                .update(("tickets", &ticket.key))
                .content(ticket)
                .await?
                .expect("tickets inserted into db");
            debug!("{:?}", tickets_insert);
        }

        Ok(self.tickets_api.issues.clone())
    }

    pub async fn get_next_ticket_page(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        self.tickets_start_at += self.tickets_max_results;
        let mut query = self.db
            .query("SELECT * FROM tickets WHERE fields.project.key = type::string($project_key) START type::number($start_at)")
            .bind(("project_key", project_key))
            .bind(("start_at", self.tickets_start_at))
            .await?;
        let tickets: Vec<TicketData> = query.take(0)?;
        if !tickets.is_empty() {
            self.tickets_api.issues = tickets;
            return Ok(self.tickets_api.issues.clone());
        }
        self.tickets_start_at -= self.tickets_max_results;

        if (self.tickets_start_at + self.tickets_max_results) < self.tickets_api.total {
            self.tickets_api.issues.clear();
            self.tickets_start_at += self.tickets_max_results;
            return self.get_and_record_tickets(project_key).await;
        }
        Ok(self.tickets_api.issues.clone())
    }

    pub async fn get_previous_tickets_page(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        self.tickets_start_at = self
            .tickets_start_at
            .saturating_sub(self.tickets_max_results);
        self.tickets_api.issues.clear();
        self.get_jira_tickets(project_key).await
    }

    pub async fn get_jira_tickets(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Vec<TicketData>, anyhow::Error> {
        let mut query = self.db
            .query("SELECT * FROM tickets WHERE fields.project.key = type::string($project_key) LIMIT type::number($limit) START type::number($start_at)")
            .bind(("project_key", project_key))
            .bind(("limit", self.tickets_max_results))
            .bind(("start_at", self.tickets_start_at))
            .await?;
        self.tickets_api.issues = query.take(0)?;
        if self.tickets_api.issues.is_empty() {
            self.get_and_record_tickets(project_key).await?;
            return Ok(self.tickets_api.issues.clone());
        }
        Ok(self.tickets_api.issues.clone())
    }

    pub async fn search_cache_ticket(
        &mut self,
        ticket_key: &str,
    ) -> anyhow::Result<TicketData, anyhow::Error> {
        let ticket: Option<TicketData> = self.db.select(("tickets", ticket_key)).await?;
        match ticket {
            Some(t) => {
                self.tickets_api.issues.push(t.clone());
                Ok(t)
            }
            None => {
                let ticket = self.jira_ticket_api(ticket_key).await?;
                Ok(ticket)
            }
        }
    }

    pub async fn jira_ticket_api(
        &mut self,
        ticket_key: &str,
    ) -> anyhow::Result<TicketData, anyhow::Error> {
        debug!("Retrieve {ticket_key}");
        let ticket = self
            .tickets_api
            .search_tickets_api(ticket_key, &self.client)
            .await?;
        self.jira_project_api(&ticket.fields.project.key).await?;
        let update_ticket_record: TicketData = self
            .db
            .update(("tickets", ticket_key))
            .content(ticket)
            .await?
            .expect("Failed to update ticket record");

        debug!("{:?}", update_ticket_record);

        Ok(update_ticket_record)
    }

    pub async fn search_cache_projects(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Project, anyhow::Error> {
        let project: Option<Project> = self.db.select(("projects", project_key)).await?;
        match project {
            Some(p) => {
                self.projects_api.values.push(p.clone());
                Ok(p)
            }
            None => self.jira_project_api(project_key).await,
        }
    }

    pub async fn jira_project_api(
        &mut self,
        project_key: &str,
    ) -> anyhow::Result<Project, anyhow::Error> {
        let project = self
            .projects_api
            .search_jira_project_api(project_key, &self.client)
            .await?;
        let update_project_record: Project = self
            .db
            .update(("projects", project_key))
            .content(project)
            .await?
            .expect("Failed to update project");

        Ok(update_project_record)
    }
}
