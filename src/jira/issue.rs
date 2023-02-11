use super::auth::JiraAuth;
use log::info;
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::Error as SurrealDbError;
use surrealdb::Surreal;

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketProject {
    key: String,
    name: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TicketStatus {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketPriority {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketIssueType {
    name: String,
    subtask: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketFields {
    issuetype: TicketIssueType,
    priority: TicketPriority,
    project: TicketProject,
    labels: Vec<String>,
    status: TicketStatus,
    summary: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketData {
    key: String,
    fields: TicketFields,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssues {
    start_at: Option<i32>,
    max_results: Option<i32>,
    total: Option<i32>,
    issues: Vec<TicketData>,
}

// TODO: handle pagination
impl JiraIssues {
    pub async fn new(jira_auth: &JiraAuth) -> anyhow::Result<Self> {
        let issues = Vec::new();
        Ok(Self {
            start_at: None,
            max_results: None,
            total: None,
            issues,
        })
    }

    async fn get_issues_from_jira_api(
        jira_auth: &JiraAuth,
        project_name: String,
    ) -> Result<String, reqwest::Error> {
        let domain = jira_auth.get_domain();
        let headers = jira_auth.get_basic_auth();
        let url = format!(
            "{}/rest/api/3/search?jql=project%20%3D%20{}",
            domain, project_name
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(url).send().await?.text().await;

        return response;
    }

    pub async fn save_jira_issues(
        db: &Surreal<Any>,
        jira_auth: &JiraAuth,
        project_name: &str,
    ) -> Result<Vec<TicketData>, SurrealDbError> {
        let resp = Self::get_issues_from_jira_api(&jira_auth, project_name.to_string())
            .await
            .expect("should be response from jira");
        let resp_slice: &str = &resp[..];
        let object: JiraIssues =
            serde_json::from_str(resp_slice).expect("unable to convert project resp to slice");
        for issues in object.issues.iter() {
            let issue_insert = db.create(("issues", &issues.key)).content(issues).await?;
            info!("{issue_insert:?}");
        }
        Ok(object.issues)
    }

    async fn get_jira_issues(
        project_key: &str,
        db: &Surreal<Any>,
    ) -> Result<Vec<TicketData>, SurrealDbError> {
        let issues: Vec<TicketData> = db.select("issues").await?;
        info!("{issues:?}");
        Ok(issues)
    }
}
