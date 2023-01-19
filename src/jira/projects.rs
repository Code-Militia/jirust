use super::auth::JiraAuth;
use log::info;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use surrealdb::sql::Value;
use surrealdb::Error as SurrealDbError;
use surrealdb::{Datastore, Session};

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
    pub key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraProjectsOutput {
    pub isLast: bool,
    pub nextPage: Option<String>,
    pub values: Vec<Project>,
}

pub struct JiraProjects<'a> {
    pub auth: &'a JiraAuth,
    pub db_connection: &'a (Datastore, Session),
}

impl<'a> JiraProjects<'_> {
    async fn get_projects_from_jira_api(&self) -> Result<String, reqwest::Error> {
        let jira_url = self.auth.get_domain();
        let jira_api_version = self.auth.get_api_version();
        let projects_url = format!("{}/rest/api/{}/project/search", jira_url, jira_api_version);
        let headers = self.auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(projects_url).send().await?.text().await;

        return response;
    }

    pub async fn save_jira_projects(&self) -> Result<(), SurrealDbError> {
        let (ds, sess) = self.db_connection;
        let resp = self
            .get_projects_from_jira_api()
            .await
            .expect("should be response from jira");
        let resp_slice: &str = &resp[..];
        let object: JiraProjectsOutput =
            serde_json::from_str(resp_slice).expect("unable to convert project resp to slice");
        for project in object.values.iter() {
            let query = format!(
                "CREATE projects:{} SET name = $name, projectId = $id, key = $key",
                project.key
            );
            let data: BTreeMap<String, Value> = [
                ("name".into(), project.name.as_str().into()),
                ("id".into(), project.id.into()),
                ("key".into(), project.key.as_str().into()),
            ]
            .into();

            let ress = ds.execute(&query, sess, Some(data), false).await?;
            info!("{ress:?}");
        }
        Ok(())
    }

    pub async fn get_jira_projects(&self) -> Result<Vec<String>, Box<dyn StdError>> {
        let (ds, sess) = self.db_connection;
        let mut resp: Vec<String> = Vec::new();
        let query = "SELECT * FROM projects;";
        let ress = ds.execute(query, &sess, None, false).await?;
        let res = ress.into_iter().next().map(|rp| rp.result).transpose()?;
        // info!("\nvalue of res {res:?}");
        match res {
            Some(Value::Array(arr)) => {
                info!("\nvalue of arr {arr:?}");
                for index in arr.iter() {
                    info!("inside arr iterator {index:?}");
                    match index {
                        Value::Object(obj) => {
                            let project = obj.get("key").unwrap().to_owned().as_string();
                            info!("project key {project:?}");
                            resp.push(project);
                        }
                        _ => (), //TODO fix this
                    }
                }
            }
            // _ => Err(Box::new(StdIoError::new(ErrorKind::Other, "value was not an array")))
            _ => (), //TODO fix this
        }
        info!("{resp:?}");
        Ok(resp)
    }
}
