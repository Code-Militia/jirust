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

    async fn get_projects_from_jira_api(
        &self,
        get_next_page: bool,
        jira_auth: &JiraClient,
    ) -> Result<String, reqwest::Error> {
        let jira_url = jira_auth.get_domain();
        let jira_api_version = jira_auth.get_api_version();
        let mut projects_url = format!("{}/rest/api/{}/project/search", jira_url, jira_api_version);
        if get_next_page && !&self.is_last {
            projects_url = match &self.next_page {
                None => projects_url,
                Some(i) => i.to_string(),
            }
        }
        let headers = jira_auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(projects_url).send().await?.text().await;

        return response;
    }

    // TODO: handle pagination
    async fn save_jira_projects(
        &mut self,
        db: &SurrealAny,
        get_next_page: bool,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<Vec<Project>> {
        let resp = self
            .get_projects_from_jira_api(get_next_page, jira_auth)
            .await?;
        let resp_slice: &str = &resp[..];
        *self = serde_json::from_str(resp_slice).expect("projects deserialized");
        for project in self.values.iter() {
            let project_insert: Project = db
                .create(("project", &project.key))
                .content(project)
                .await?;
            info!("{project_insert:?}");
        }
        Ok(self.values.clone())
    }

    pub async fn get_jira_projects(
        &mut self,
        db: &SurrealAny,
        get_next_page: bool,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<Vec<Project>> {
        let projects: Vec<Project> = db.select("project").await?;
        if projects.is_empty() {
            return Ok(self
                .save_jira_projects(db, get_next_page, jira_auth)
                .await?);
        }
        info!("{projects:?}");
        Ok(projects)
    }
}
