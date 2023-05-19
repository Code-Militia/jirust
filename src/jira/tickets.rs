use super::auth::JiraClient;
use super::SurrealAny;
use htmltoadf::convert_html_str_to_adf_str;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkFields {
    pub issuetype: Type,
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
    pub issuetype: Type,
    pub issuelinks: Vec<Links>,
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
pub struct TicketTransition {
    pub id: String,
    pub name: Option<String>,
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
        let _db_update: TicketData = db.update(("tickets", &self.key)).merge(&self).await?;
        Ok(comments)
    }

    pub async fn get_comments(
        &self,
        db: &SurrealAny,
        jira_client: &JiraClient,
    ) -> anyhow::Result<Comments> {
        let ticket: TicketData = db
            .select(("tickets", &self.key))
            .await
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

        let _db_update: TicketData = db.update(("tickets", &self.key)).merge(&self).await?;
        Ok(comments)
    }

    pub async fn get_transitions(
        &self,
        jira_client: &JiraClient,
    ) -> anyhow::Result<TicketTransitions> {
        let url = format!("/issue/{}/transitions", self.key);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: TicketTransitions = serde_json::from_str(&response)?;
        Ok(obj)
    }

    pub async fn transition(
        &self,
        transition: PostTicketTransition,
        jira_client: &JiraClient,
    ) -> anyhow::Result<()> {
        let url = format!("/issue/{}/transitions", self.key);
        let data = serde_json::to_string(&transition)?;
        jira_client.post_to_jira_api(&url, data).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraTickets {
    pub start_at: Option<u32>,
    pub max_results: Option<u32>,
    pub total: u32,
    pub issues: Vec<TicketData>,
}

impl JiraTickets {
    pub async fn new() -> anyhow::Result<Self> {
        let issues = Vec::new();
        Ok(Self {
            start_at: None,
            max_results: None,
            total: 0,
            issues,
        })
    }

    pub async fn get_tickets_from_jira_api(
        &self,
        jira_auth: &JiraClient,
        params: Vec<(&str, &str)>,
        url: &str,
    ) -> Result<String, reqwest::Error> {
        let headers = jira_auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        client
            .get(url)
            .query(&params)
            .send()
            .await?
            .text().
            await
    }

    pub async fn search_jira_ticket_api(
        &self,
        ticket_key: &str,
        jira_client: &JiraClient,
    ) -> anyhow::Result<TicketData> {
        let url = format!("/issue/{}?expand=renderedFields", ticket_key);
        let response = jira_client.get_from_jira_api(&url).await?;
        let obj: TicketData = serde_json::from_str(&response)?;
        Ok(obj)
    }
}
