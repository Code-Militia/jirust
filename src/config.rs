use std::env;

// use crate::{event::key::Key, log::LogLevel};
use crate::event::key::Key;

use serde::Deserialize;

#[cfg(test)] // TODO: What does this do?
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub key_config: KeyConfig,
    pub jira_config: JiraConfig,
    // #[serde(default)]
    // pub log_level: LogLevel,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct JiraConfig {
    pub api_key: String,
    pub api_version: String,
    pub domain: String,
    pub user_email: String,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct KeyConfig {
    pub enter: Key,
    pub esc: Key,
    pub exit: Key,
    pub filter: Key,
    pub filter_edit: Key,
    pub move_up: Key,
    pub move_down: Key,
    pub move_left: Key,
    pub move_right: Key,
    pub open_browser: Key,
    pub open_help: Key,
    pub next_page: Key,
    pub previous_page: Key,
    pub quit: Key,
    pub scroll_up: Key,
    pub scroll_down: Key,
    pub scroll_right: Key,
    pub scroll_left: Key,
    pub scroll_down_multiple_lines: Key,
    pub scroll_up_multiple_lines: Key,
    pub scroll_to_top: Key,
    pub scroll_to_bottom: Key,
    pub ticket_transition: Key,
    pub ticket_add_comments: Key,
    pub ticket_view_comments: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            enter: Key::Enter,
            esc: Key::Esc,
            exit: Key::Ctrl('c'),
            filter: Key::Char('/'),
            filter_edit: Key::Char('e'),
            move_up: Key::Up,
            move_down: Key::Down,
            move_left: Key::Left,
            move_right: Key::Right,
            next_page: Key::Char('n'),
            open_browser: Key::Char('o'),
            open_help: Key::Char('?'),
            previous_page: Key::Char('N'),
            quit: Key::Char('q'),
            scroll_up: Key::Char('k'),
            scroll_down: Key::Char('j'),
            scroll_right: Key::Char('l'),
            scroll_left: Key::Char('h'),
            scroll_down_multiple_lines: Key::Ctrl('d'),
            scroll_up_multiple_lines: Key::Ctrl('u'),
            scroll_to_top: Key::Char('g'),
            scroll_to_bottom: Key::Char('G'),
            ticket_transition: Key::Char('t'),
            ticket_add_comments: Key::Char('C'),
            ticket_view_comments: Key::Char('c'),
        }
    }
}

impl Default for JiraConfig {
    fn default() -> Self {
        let jira_api_key = match env::var("JIRA_API_KEY") {
            Ok(v) => v,
            Err(_e) => {
                panic!("Environment variable JIRA_API_KEY is not set")
            }
        };

        let jira_api_version = match env::var("JIRA_API_VERSION") {
            Ok(v) => v,
            Err(_e) => {
                panic!("Environment variable JIRA_API_VERSION is not set")
            }
        };

        let jira_domain = match env::var("JIRA_DOMAIN") {
            Ok(v) => v,
            Err(_e) => {
                panic!("Environment variable JIRA_DOMAIN is not set")
            }
        };

        let jira_user_email = match env::var("JIRA_USER_EMAIL") {
            Ok(v) => v,
            Err(_e) => {
                panic!("Environment variable JIRA_USER_EMAIL is not set")
            }
        };
        Self {
            api_key: jira_api_key,
            api_version: jira_api_version,
            domain: jira_domain,
            user_email: jira_user_email,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_config: KeyConfig::default(),
            jira_config: JiraConfig::default(),
        }
    }
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Config::default())
    }
}
