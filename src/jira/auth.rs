use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JiraClient {
    pub jira_api_key: String,
    pub jira_api_version: String,
    pub jira_email: String,
    pub jira_url: String,
}

impl JiraClient {
    // pub fn set_domain(&mut self, url: String) {
    //     self.jira_url = url
    // }
    //
    // pub fn set_api_version(&mut self, api_version: String) {
    //     self.jira_api_version = api_version
    // }
    //
    // pub fn set_api_key(&mut self, jira_api_key: String) {
    //     self.jira_api_key = jira_api_key
    // }

    pub fn get_basic_auth(&self) -> HeaderMap {
        let header_content_type = HeaderValue::from_static("application/json");
        let jira_basic_auth_str = format!("Basic {}", self.jira_api_key);
        let mut jira_token_header = HeaderValue::from_str(&jira_basic_auth_str).unwrap();
        jira_token_header.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, header_content_type.clone());
        headers.insert(ACCEPT, header_content_type);
        headers.insert(AUTHORIZATION, jira_token_header);

        headers
    }

    pub fn get_domain(&self) -> &String {
        &self.jira_url
    }
    pub fn get_api_version(&self) -> &String {
        &self.jira_api_version
    }

    pub async fn post_to_jira_api(&self, api_url: &str, data: String) -> anyhow::Result<String> {
        let headers = self.get_basic_auth();
        let api_url = format!("{}/{}", self.get_domain(), api_url);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.post(api_url).body(data).send().await?.text().await?;
        Ok(response)
    }

    pub async fn get_from_jira_api(&self, api_url: &str) -> anyhow::Result<String> {
        let headers = self.get_basic_auth();
        let api_url = format!("{}/{}", self.get_domain(), api_url);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        let response = client.get(api_url).send().await?.text().await?;
        Ok(response)
    }

    pub fn new(
        jira_api_version: String,
        jira_api_key: String,
        jira_email: String,
        jira_url: String,
    ) -> Self {
        JiraClient {
            jira_api_key,
            jira_api_version,
            jira_email,
            jira_url,
        }
    }
}

pub fn jira_authentication(
    jira_domain: &str,
    jira_api_key: &str,
    jira_api_version: &str,
    jira_user_email: &str,
) -> JiraClient {
    let jira_encoded_auth: String =
        general_purpose::STANDARD_NO_PAD.encode(format!("{jira_user_email}:{jira_api_key}"));
    JiraClient::new(
        jira_api_version.to_string(),
        jira_encoded_auth,
        jira_user_email.to_string(),
        jira_domain.to_string(),
    )
}
