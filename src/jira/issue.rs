use super::auth::JiraAuth;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use surrealdb::sql::Value;
use surrealdb::Error as SurrealDbError;
use surrealdb::{Datastore, Session};

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketProject {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: i32,
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
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
    key: String,
    fields: TicketFields,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraIssuesOutput {
    startAt: i32,
    maxResults: i32,
    total: i32,
    issues: Vec<TicketData>,
}

pub struct JiraIssues<'a> {
    pub auth: &'a JiraAuth,
    pub db_connection: &'a (Datastore, Session),
}

// TODO: handle pagination
impl<'a> JiraIssues<'_> {
    async fn get_issues_for_project(&self, project_name: String) -> Result<String, reqwest::Error> {
        let domain = self.auth.get_domain();
        let headers = self.auth.get_basic_auth();
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

    pub async fn save_jira_issues(&self, project_name: &str) -> Result<(), SurrealDbError> {
        let (ds, sess) = self.db_connection;
        let resp = self
            .get_issues_for_project(project_name.to_string())
            .await
            .expect("should be response from jira");
        let resp_slice: &str = &resp[..];
        let object: JiraIssuesOutput =
            serde_json::from_str(resp_slice).expect("unable to convert project resp to slice");
        for issues in object.issues.iter() {
            let query = format!(
                "
                CREATE issues CONTENT {{
                    type: $issue_type,
                    issueId: $id,
                    key: $key,
                    summary: $summary,
                    project: project:{},
                    priority: $priority,
                    status: $status,
                    ticketProjectId: $project_id,
                    ticketProjectName: $project_name,
                    ticketProjectKey: $project_key
                }};",
                project_name
            );
            let data: BTreeMap<String, Value> = [
                ("type".into(), issues.fields.issuetype.name.as_str().into()),
                ("project_name".into(), project_name.into()),
                ("id".into(), issues.id.into()),
                ("key".into(), issues.key.as_str().into()),
                ("summary".into(), issues.fields.summary.as_str().into()),
                ("status".into(), issues.fields.status.name.as_str().into()),
                (
                    "priority".into(),
                    issues.fields.priority.name.as_str().into(),
                ),
                ("project_id".into(), issues.fields.project.id.into()),
                (
                    "project_name".into(),
                    issues.fields.project.name.as_str().into(),
                ),
                (
                    "project_key".into(),
                    issues.fields.project.key.as_str().into(),
                ),
            ]
            .into();

            let ress = ds.execute(&query, &sess, Some(data), false).await?;
        }
        // todo!("Return query errors when we run into them");
        Ok(())
    }

    async fn query_jira_issues_from_db(
        &self,
        project_key: &str,
    ) -> Result<Vec<String>, SurrealDbError> {
        let (ds, sess) = self.db_connection;
        let mut resp: Vec<String> = Vec::new();
        let query = format!(
            "SELECT * FROM issues WHERE ticketProjectKey = '{}'",
            project_key
        );
        let ress = ds.execute(&query, &sess, None, false).await?;
        let res = ress.into_iter().next().map(|rp| rp.result).transpose()?;
        match res {
            Some(Value::Array(arr)) => {
                for index in arr.iter() {
                    match index {
                        Value::Object(obj) => {
                            let issue = obj.get("key").unwrap().to_owned().as_string();
                            resp.push(issue);
                        }
                        _ => (), //TODO fix this
                    }
                }
            }
            // _ => Err(Box::new(StdIoError::new(ErrorKind::Other, "value was not an array")))
            _ => (), //TODO fix this
        }
        Ok(resp)
    }

    pub async fn get_jira_issues(
        &self,
        project_key: &str,
    ) -> Result<Vec<String>, Box<dyn StdError>> {
        let query_from_db = self
            .query_jira_issues_from_db(project_key)
            .await
            .expect("return an array from db");
        if query_from_db.len() == 0 {
            self.save_jira_issues(project_key).await;
        }
        Ok(self
            .query_jira_issues_from_db(project_key)
            .await
            .expect("return an array from db"))
    }
}
