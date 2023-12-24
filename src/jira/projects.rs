use super::auth::JiraClient;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    #[serde(alias = "id")]
    pub project_id: String,
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraProjectsAPI {
    pub is_last: bool,
    pub max_results: u32,
    pub next_page: Option<String>,
    pub start_at: u32,
    pub total: u32,
    pub values: Vec<Project>,
}

impl JiraProjectsAPI {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            is_last: true,
            max_results: 0,
            next_page: None,
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
        client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn get_projects_next_page(
        &self,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<JiraProjectsAPI> {
        match &self.next_page {
            None => Ok(self.clone()),
            Some(next_page_url) => {
                let resp = self
                    .get_projects_from_jira_api(jira_auth, next_page_url.to_string())
                    .await?;
                let object: JiraProjectsAPI = serde_json::from_str(resp.as_str())?;
                Ok(object)
            }
        }
    }

    pub async fn search_jira_project_api(
        &self,
        project_key: &str,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Project> {
        let url = format!("project/{}", project_key);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: Project = serde_json::from_str(&response)?;
        debug!("Projects from JIRA: {:?}", obj);
        Ok(obj)
    }
}
