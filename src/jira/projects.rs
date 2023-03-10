use super::auth::JiraClient;
use super::SurrealAny;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraProjects {
    pub is_last: Option<bool>,
    pub next_page: Option<String>,
    pub values: Vec<Project>,
}

impl JiraProjects {
    pub async fn new(jira_auth: &JiraClient, db: &SurrealAny) -> anyhow::Result<Self> {
        let projects = Self::get_jira_projects(db, jira_auth).await?;
        Ok(Self {
            is_last: Some(true), // TODO: Will need to refactor to handle pagination
            next_page: None,     // TODO: Will need to refactor to handle pagination
            values: projects,
        })
    }

    async fn get_projects_from_jira_api(jira_auth: &JiraClient) -> Result<String, reqwest::Error> {
        let jira_url = jira_auth.get_domain();
        let jira_api_version = jira_auth.get_api_version();
        let projects_url = format!("{}/rest/api/{}/project/search", jira_url, jira_api_version);
        let headers = jira_auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(projects_url).send().await?.text().await;
        info!("response from jira -- {:?}", response);

        return response;
    }

    // TODO: handle pagination
    pub async fn save_jira_projects(
        db: &SurrealAny,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<Vec<Project>> {
        let resp = Self::get_projects_from_jira_api(jira_auth).await?;
        let resp_slice: &str = &resp[..];
        let object: JiraProjects =
            serde_json::from_str(resp_slice).expect("unable to convert project resp to slice");
        for project in object.values.iter() {
            let project_insert: Project = db
                .create(("project", &project.key))
                .content(project)
                .await?;
            info!("{project_insert:?}");
        }
        Ok(object.values)
    }

    pub async fn get_jira_projects(
        db: &SurrealAny,
        jira_auth: &JiraClient,
    ) -> anyhow::Result<Vec<Project>> {
        let projects: Vec<Project> = db.select("project").await?;
        if projects.is_empty() {
            return Ok(Self::save_jira_projects(db, jira_auth).await?);
        }
        info!("{projects:?}");
        Ok(projects)
    }
}
