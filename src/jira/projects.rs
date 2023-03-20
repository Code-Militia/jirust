use super::auth::JiraClient;
use super::SurrealAny;
use log::info;
use serde::{Deserialize, Serialize};

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

    pub async fn get_projects_from_jira_api(
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
}
