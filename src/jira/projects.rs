use log::info;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::collections::BTreeMap;
use surrealdb::sql::Value;
use surrealdb::Error as SurrealDbError;
use super::Jira;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
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

// for projects
impl Jira {}

// for issues
impl Jira {}

// For next time, change save projects to build out Projects struct and call save to put into db

impl JiraProjects {
    pub async fn new(jira: &Jira) -> anyhow::Result<Self> {
        let jp = Self::default();
        Self::save_jira_projects(jira);
        let projects = jp.get_jira_projects(jira).await?;
        Ok(Self {
            is_last: Some(true), // TODO: Will need to refactor to handle pagination
            next_page: None, // TODO: Will need to refactor to handle pagination
            values: projects
        })
    }

    async fn get_projects_from_jira_api(jira: &Jira) -> Result<String, reqwest::Error> {
        let jira_url = jira.auth.get_domain();
        let jira_api_version = jira.auth.get_api_version();
        let projects_url = format!("{}/rest/api/{}/project/search", jira_url, jira_api_version);
        let headers = jira.auth.get_basic_auth();
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(projects_url).send().await?.text().await;

        return response;
    }

    pub async fn save_jira_projects(jira: &Jira) -> anyhow::Result<()> {
        let (ds, sess) = &jira.db;
        let resp = Self::get_projects_from_jira_api(jira)
            .await?;
        let resp_slice: &str = &resp[..];
        let object: JiraProjects =
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

    pub async fn get_jira_projects(&self, jira: &Jira) -> anyhow::Result<Vec<Project>> {
        let (ds, sess) = &jira.db;
        let mut resp = Vec::new(); 
        let query = "SELECT * FROM projects;";
        let ress = ds.execute(query, &sess, None, false).await?;
        let res = ress.into_iter().next().map(|rp| rp.result).transpose()?;
        match res {
            Some(Value::Array(arr)) => {
                info!("\nvalue of arr {arr:?}");
                for index in arr.iter() {
                    info!("inside arr iterator {index:?}");
                    match index {
                        Value::Object(obj) => {
                            let project_key = obj.get("key").unwrap().to_owned().as_string();
                            let project_id = obj.get("id").unwrap().to_owned().as_string();
                            let project_name = obj.get("name").unwrap().to_owned().as_string();
                            info!("project key {project_key:?}");
                            resp.push(Project { id: project_id.parse().unwrap(), key: project_key, name: project_name });
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
