use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub account_id: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JiraClient {
    pub api_key: String,
    pub api_version: String,
    pub email: String,
    pub url: String,
    pub user: Option<UserData>,
}

impl JiraClient {
    pub fn get_basic_auth(&self) -> HeaderMap {
        let header_content_type = HeaderValue::from_static("application/json");
        let jira_basic_auth_str = format!("Basic {}", self.api_key);
        let mut jira_token_header = HeaderValue::from_str(&jira_basic_auth_str).unwrap();
        jira_token_header.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, header_content_type.clone());
        headers.insert(ACCEPT, header_content_type);
        headers.insert(AUTHORIZATION, jira_token_header);

        headers
    }

    pub fn get_domain(&self) -> &String {
        &self.url
    }

    pub async fn post_to_jira_api(
        &self,
        api_url: &str,
        data: Option<String>,
    ) -> anyhow::Result<String> {
        let headers = self.get_basic_auth();
        let api_url = format!("{}/{}", self.get_domain(), api_url);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(true)
            .build()?;
        match data {
            Some(d) => {
                let response = client
                    .post(api_url.clone())
                    .body(d.clone())
                    .send()
                    .await?
                    .text()
                    .await?;
                debug!("client {:#?}", client);
                debug!("api url {}", api_url);
                debug!("api response {} ", response);
                Ok(response)
            }
            None => {
                let response = client.post(api_url.clone()).send().await?.text().await?;
                debug!("client {:#?}", client);
                debug!("api url {}", api_url);
                debug!("api response {} ", response);
                Ok(response)
            }
        }
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

    pub async fn new(
        jira_api_version: String,
        jira_api_key: String,
        jira_email: String,
        jira_url: String,
    ) -> Self {
        let mut client = JiraClient {
            api_key: jira_api_key,
            api_version: jira_api_version,
            email: jira_email,
            url: jira_url,
            user: None,
        };
        let current_user = get_current_user(&client);
        match current_user.await {
            Ok(x) => {
                debug!("{:#?}", x);
                client.user = Some(x[0].clone())
            }
            Err(error) => debug!("unable to find user data {}", error),
        };
        client
    }
}

// Get current user details
async fn get_current_user(jira_client: &JiraClient) -> anyhow::Result<Vec<UserData>> {
    debug!("Getting current user");
    let url: String = format!("user/search?query={}", jira_client.email);
    let data = jira_client.get_from_jira_api(&url).await?;
    debug!("user data from POST {:#?}", data);
    let obj: Vec<UserData> = serde_json::from_str(&data)?;
    if obj.is_empty() {
        return Err(anyhow!("unable to locate user data"));
    }
    Ok(obj)
}

pub async fn jira_authentication(
    jira_domain: &str,
    jira_api_key: &str,
    jira_api_version: &str,
    jira_user_email: &str,
) -> JiraClient {
    let jira_encoded_auth: String =
        general_purpose::URL_SAFE.encode(format!("{jira_user_email}:{jira_api_key}"));
    let jira_rest_domain = jira_domain.to_string() + "/rest/api/" + jira_api_version;
    JiraClient::new(
        jira_api_version.to_string(),
        jira_encoded_auth,
        jira_user_email.to_string(),
        jira_rest_domain,
    )
    .await
}
