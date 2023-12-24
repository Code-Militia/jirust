use std::collections::HashMap;

use super::auth::JiraClient;
use super::SurrealAny;
use htmltoadf::convert_html_str_to_adf_str;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkFields {
    pub issuetype: TicketType,
    pub priority: Option<Priority>,
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
    // #[serde(alias = "self")]
    // pub parent_self: String
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
pub struct ProjectDetails {
    pub key: String,
    pub name: String,
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
pub struct TicketType {
    pub id: String,
    pub name: String,
    pub subtask: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldAuthor {
    pub display_name: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommentBody {
    pub author: FieldAuthor,
    pub created: String,
    pub rendered_body: String,
    pub updated: String,
    pub update_author: FieldAuthor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comments {
    pub comments: Vec<CommentBody>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fields {
    pub assignee: Option<Assignee>,
    pub comments: Option<Comments>,
    pub components: Vec<Components>,
    pub creator: Option<CreatorReporter>,
    pub issuelinks: Vec<Links>,
    pub issuetype: TicketType,
    pub labels: Vec<String>,
    pub parent: Option<LinkInwardOutwardParent>,
    pub priority: Option<Priority>,
    pub project: ProjectDetails,
    pub reporter: Option<CreatorReporter>,
    pub status: Status,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketData {
    pub fields: Fields,
    pub key: String,
    pub rendered_fields: RenderedFields,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldAllowedValues {
    pub id: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldSchema {
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldValues {
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub name: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub key: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<CustomFieldSchema>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<CustomFieldAllowedValues>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    #[serde(flatten)]
    pub values: Option<HashMap<String, CustomFieldValues>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketTransition {
    pub id: String,
    pub name: Option<String>,
    pub has_screen: Option<bool>,
    pub fields: Option<CustomField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketTransitions {
    pub transitions: Vec<TicketTransition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostTicketTransition {
    pub transition: TicketTransition,
}

impl TicketData {
    async fn save_ticket_comments_from_api(
        &self,
        db: &SurrealAny,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Comments> {
        let url = format!("/issue/{}/comment?expand=renderedBody", self.key);
        let response = jira_client.get_from_jira_api(&url).await?;

        let comments: Comments =
            serde_json::from_str(response.as_str()).expect("unable to deserialize comments");
        let _db_update: TicketData = db
            .update(("tickets", &self.key))
            .merge(&self)
            .await?
            .expect("Failed to update tickets");
        Ok(comments)
    }

    pub async fn get_comments(
        &self,
        db: &SurrealAny,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Comments> {
        let ticket: TicketData = db
            .select(("tickets", &self.key))
            .await?
            .expect("Failed to get TicketData from DB in get_comments");
        match ticket.fields.comments {
            None => Ok(self.save_ticket_comments_from_api(db, jira_client).await?),
            Some(c) => Ok(c),
        }
    }

    pub async fn add_comment(
        &self,
        db: &SurrealAny,
        comment: &str,
        jira_client: &JiraClient,
    ) -> anyhow::Result<CommentBody> {
        let url = format!("/issue/{}/comment?expand=renderedBody", self.key);
        let html = markdown::to_html(comment);
        let adf = convert_html_str_to_adf_str(html);
        let adf = format!("{{ \"body\": {} }}", adf);
        let response = jira_client
            .post_to_jira_api(&url, adf)
            .await
            .expect("unable to save comment");
        let comments: CommentBody =
            serde_json::from_str(response.as_str()).expect("unable to deserialize comments");

        let _db_update: TicketData = db
            .update(("tickets", &self.key))
            .merge(&self)
            .await?
            .expect("Failed to update tickets");
        Ok(comments)
    }

    pub async fn get_transitions(
        &self,
        jira_client: &JiraClient,
    ) -> anyhow::Result<TicketTransitions> {
        let url = format!("/issue/{}/transitions?expand=transitions.fields", self.key);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: TicketTransitions = serde_json::from_str(&response)?;
        debug!("Ticket transitions {:?}", obj);
        Ok(obj)
    }

    pub async fn transition_ticket(
        &self,
        transition: PostTicketTransition,
        jira_client: &JiraClient,
    ) -> anyhow::Result<String> {
        let url = format!("/issue/{}/transitions", self.key);
        let data = serde_json::to_string(&transition)?;
        let post = jira_client.post_to_jira_api(&url, data).await?;
        Ok(post)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicket {
    pub description: String,
    pub project_id: String,
    pub summary: String,
    pub ticket_types: Vec<TicketType>,
}

impl CreateTicket {
    pub fn new() -> Self {
        Self {
            description: String::new(),
            project_id: String::new(),
            summary: String::new(),
            ticket_types: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraTicketsAPI {
    pub start_at: Option<u32>,
    pub max_results: Option<u32>,
    pub total: u32,
    pub issues: Vec<TicketData>,
}

impl JiraTicketsAPI {
    pub async fn new() -> anyhow::Result<Self> {
        let issues = Vec::new();
        Ok(Self {
            start_at: None,
            max_results: None,
            total: 0,
            issues,
        })
    }

    pub async fn get_tickets_api(
        &self,
        jira_client: &JiraClient,
        params: Vec<(&str, &str)>,
        url: &str,
    ) -> Result<String, reqwest::Error> {
        let headers = jira_client.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        client.get(url).query(&params).send().await?.text().await
    }

    pub async fn search_tickets_api(
        &self,
        ticket_key: &str,
        jira_client: &JiraClient,
    ) -> anyhow::Result<TicketData> {
        let url = format!("/issue/{}?expand=renderedFields", ticket_key);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: TicketData = serde_json::from_str(&response)?;
        Ok(obj)
    }

    pub async fn get_ticket_types(
        &self,
        jira_client: &JiraClient,
        project_id: &str,
    ) -> anyhow::Result<Vec<TicketType>> {
        let url = format!("/issuetype/project?projectId={}", project_id);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: Vec<TicketType> = serde_json::from_str(&response)?;
        Ok(obj)
    }

    pub async fn create_ticket_api(
        &self,
        jira_client: &JiraClient,
        create_ticket_data: CreateTicket,
    ) -> anyhow::Result<()> {
        let url = String::from("rest/api/3/issue");
        let data = serde_json::to_string(&create_ticket_data)?;
        todo!("Format data correctly"); // See https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issues/#api-rest-api-3-issue-post
        jira_client.post_to_jira_api(&url, data).await?;
        Ok(())
    }
}
