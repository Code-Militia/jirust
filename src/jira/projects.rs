use std::{rc::Rc, thread};

use super::auth::JiraClient;
use super::SurrealAny;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::{task::{spawn_local, self}, runtime::Runtime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraProjects {
    pub is_last: bool,
    pub max_results: u32,
    pub next_page: Option<String>,
    pub start_at: u32,
    pub total: u32,
    pub values: Vec<Project>,
}

impl JiraProjects {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            is_last: true, // TODO: Will need to refactor to handle pagination
            max_results: 0,
            next_page: None, // TODO: Will need to refactor to handle pagination
            start_at: 0,
            total: 0,
            values: Vec::new(),
        })
    }

    async fn get_projects_from_jira_api(
        &self,
        jira_auth: &JiraClient,
        url: String,
    ) -> Result<String, reqwest::Error> {
        let headers = jira_auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(url).send().await?.text().await;

        return response;
    }

    pub async fn get_jira_projects(
        & mut self,
        db: &SurrealAny,
        get_next_page: bool,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<Vec<Project>> {

        // TODO: Need a way to get previous page
        // TODO: This always gets it from the API.  This needs to try to get from DB first
        match &self.next_page {
            None => {},
            Some(url) if get_next_page && !&self.is_last => {
                let resp = self.get_projects_from_jira_api(jira_auth, url.to_string()).await?;
                *self = serde_json::from_str(resp.as_str()).expect("projects deserialized");
                return Ok(self.values.clone())
            },
            &Some(_) => {
                info!("we're in this block");
                todo!("need to handle cases for wildcards")
            }
        }

        let projects: Vec<Project> = db.select("project").await?;
        let jira_url = jira_auth.get_domain();
        let jira_api_version = jira_auth.get_api_version();
        if projects.is_empty() {
            let url = format!("{}/rest/api/{}/project/search?maxResults=1&startAt=0", jira_url, jira_api_version);
            let resp = self.get_projects_from_jira_api(jira_auth, url).await?;
            *self = serde_json::from_str(resp.as_str()).expect("projects deserialized");
            let values = self.values.clone();

            // If the database is ever going to be something other than embedded
            // it's best if we move the create method to another thread.
            let _projects_insert: JiraProjects = db
                .create("project")
                .content(self)
                .await.expect("projects inserted into db");
            return Ok(values)
        }

        info!("{projects:?}");
        Ok(projects)
    }
}
