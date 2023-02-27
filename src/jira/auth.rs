use base64::{Engine as _, engine::general_purpose};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
#[derive(Debug)]
pub struct JiraAuth {
    pub jira_api_key: String,
    pub jira_api_version: String,
    pub jira_email: String,
    pub jira_url: String,
}

impl JiraAuth {
    pub fn set_domain(&mut self, url: String) {
        self.jira_url = url
    }

    pub fn set_api_version(&mut self, api_version: String) {
        self.jira_api_version = api_version
    }

    pub fn set_api_key(&mut self, jira_api_key: String) {
        self.jira_api_key = jira_api_key
    }

    pub fn get_basic_auth(&self) -> HeaderMap {
        let header_content_type = HeaderValue::from_static("application/json");
        let jira_basic_auth_str = format!("Basic {}", self.jira_api_key).to_string();
        let mut jira_token_header = HeaderValue::from_str(&jira_basic_auth_str).unwrap();
        jira_token_header.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, header_content_type);
        headers.insert(AUTHORIZATION, jira_token_header);

        return headers;
    }

    pub fn get_domain(&self) -> &String {
        return &self.jira_url;
    }
    pub fn get_api_version(&self) -> &String {
        return &self.jira_api_version;
    }

    pub fn new(jira_api_version: String, jira_api_key: String, jira_email: String, jira_url: String) -> Self {
        return JiraAuth {
            jira_api_key,
            jira_api_version,
            jira_email,
            jira_url,
        };
    }
}

pub fn jira_authentication() -> JiraAuth {
    let env_jira_url = "JIRA_URL";
    let env_jira_api_version = "JIRA_API_VERSION";
    let env_jira_api_key = "JIRA_API_KEY";
    let env_jira_email = "JIRA_EMAIL";
    let jira_url = env::var(env_jira_url).expect("$JIRA_URL is not set");
    let jira_api_version = env::var(env_jira_api_version).expect("$JIRA_API_VERSION is not set");
    let jira_api_key = env::var(env_jira_api_key).expect("$JIRA_API_KEY is not set");
    let jira_email = env::var(env_jira_email).expect("$JIRA_EMAIL is not set");
    let jira_encoded_auth: String = general_purpose::STANDARD_NO_PAD.encode(format!("{jira_email}:{jira_api_key}"));
    return JiraAuth::new(jira_api_version, jira_encoded_auth, jira_email, jira_url);
}
