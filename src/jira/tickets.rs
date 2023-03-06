use super::auth::JiraAuth;
use super::SurrealAny;
use log::info;
use serde::{Deserialize, Serialize};
use surrealdb::Error as SurrealDbError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkFields {
    pub issuetype: Type,
    pub priority: Priority,
    pub status: Status,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkType {
    pub inward: String,
    pub outward: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkInwardOutwardParent {
    pub fields: LinkFields,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub inward_issue: Option<LinkInwardOutwardParent>,
    pub outward_issue: Option<LinkInwardOutwardParent>,
    #[serde(alias = "type")]
    pub link_type: LinkType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatorReporter {
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Assignee {
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderedFields {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Components {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    key: String,
    name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Status {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Priority {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Type {
    pub name: String,
    pub subtask: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub components: Vec<Components>,
    pub creator: Option<CreatorReporter>,
    pub issuetype: Type,
    pub issuelinks: Vec<Links>,
    pub labels: Vec<String>,
    pub parent: Option<LinkInwardOutwardParent>,
    pub priority: Option<Priority>,
    pub project: Project,
    pub reporter: Option<CreatorReporter>,
    pub status: Status,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TicketData {
    pub fields: Fields,
    pub key: String,
    pub rendered_fields: RenderedFields,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JiraTickets {
    start_at: Option<i32>,
    max_results: Option<i32>,
    total: Option<i32>,
    issues: Vec<TicketData>,
}

// TODO: handle pagination
impl JiraTickets {
    pub async fn new() -> anyhow::Result<Self> {
        let issues = Vec::new();
        Ok(Self {
            start_at: None,
            max_results: None,
            total: None,
            issues,
        })
    }

    async fn get_tickets_from_jira_api(
        jira_auth: &JiraAuth,
        project_name: String,
    ) -> Result<String, reqwest::Error> {
        let domain = jira_auth.get_domain();
        let headers = jira_auth.get_basic_auth();
        let url = format!(
            "{}/rest/api/3/search?jql=project%20%3D%20{}&expand=renderedFields",
            domain, project_name
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(url).send().await?.text().await;

        return response;
    }

    pub async fn save_jira_tickets(
        db: &SurrealAny,
        jira_auth: &JiraAuth,
        project_key: &str,
    ) -> Result<Vec<TicketData>, SurrealDbError> {
        let resp = Self::get_tickets_from_jira_api(&jira_auth, project_key.to_string())
            .await
            .expect("should be response from jira");
        let resp_slice: &str = &resp[..];
        let object: JiraTickets =
            serde_json::from_str(resp_slice).expect("unable to convert project resp to slice");
        for ticket in object.issues.iter() {
            let issue_insert: TicketData =
                db.create(("tickets", &ticket.key)).content(&ticket).await?;
            info!("Creating ticket inside db -- {issue_insert:?}");
        }
        Ok(object.issues)
    }

    pub async fn get_jira_tickets(
        &self,
        db: &SurrealAny,
        jira_auth: &JiraAuth,
        project_key: &str,
    ) -> Result<Vec<TicketData>, SurrealDbError> {
        let tickets: Vec<TicketData> = db.select("tickets").await?;
        if tickets.is_empty() {
            return Ok(Self::save_jira_tickets(db, jira_auth, project_key).await?);
        }
        info!("{tickets:?}");
        Ok(tickets)
    }
}
