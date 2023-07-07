use std::{env, fs, process::exit};

// use crate::{event::key::Key, log::LogLevel};
use crate::event::key::Key;

use serde::Deserialize;

#[cfg(test)] // TODO: What does this do?
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct JiraConfigFile {
    pub api_key: Option<String>,
    pub api_version: Option<String>,
    pub db_file: Option<bool>,
    pub domain: String,
    pub user_email: String,
    pub projects: Option<JiraConfigProjects>,
    pub tickets: Option<JiraConfigTickets>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JiraConfigTickets {
    pub current_user_tickets_only: Option<bool>,
    pub show_unassgined: Option<bool>,
    pub show_ticket_status: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JiraConfigProjects {
    pub default_projects: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub key_config: KeyConfig,
    pub jira_config: JiraConfigFile,
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
    pub next: Key,
    pub next_page: Key,
    pub previous: Key,
    pub previous_page: Key,
    pub quit: Key,
    pub reset: Key,
    pub scroll_up: Key,
    pub scroll_down: Key,
    pub page_up: Key,
    pub page_down: Key,
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
            next: Key::Tab,
            next_page: Key::Char('n'),
            open_browser: Key::Char('o'),
            open_help: Key::Char('?'),
            page_up: Key::Char('K'),
            page_down: Key::Char('J'),
            previous: Key::BackTab,
            previous_page: Key::Char('N'),
            quit: Key::Char('q'),
            reset: Key::Char('r'),
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

impl Default for JiraConfigFile {
    fn default() -> Self {
        let home_directory = env!("HOME");
        let filename = format!("{}/.config/jirust/config.toml", home_directory);
        let contents = match fs::read_to_string(filename.clone()) {
            // If successful return the files text as `contents`.
            // `c` is a local variable.
            Ok(c) => c,
            // Handle the `error` case.
            Err(_) => {
                // Write `msg` to `stderr`.
                eprintln!("Could not read file `{}`", filename);
                // Exit the program with exit code `1`.
                exit(1);
            }
        };

        // Use a `match` block to return the
        // file `contents` as a `Data struct: Ok(d)`
        // or handle any `errors: Err(_)`.
        let data: JiraConfigFile = match toml::from_str(&contents) {
            // If successful, return data as `Data` struct.
            // `d` is a local variable.
            Ok(d) => d,
            // Handle the `error` case.
            Err(e) => {
                // Write `msg` to `stderr`.
                eprintln!("Unable to load data from `{}` - {}", filename, e);
                // Exit the program with exit code `1`.
                exit(1);
            }
        };

        let db_file = data.db_file;
        let domain = data.domain;
        let jira_user_email = data.user_email;

        let jira_api_key = match env::var("JIRA_API_KEY") {
            Ok(v) => v,
            Err(_e) => {
                panic!("Environment variable JIRA_API_KEY is not set")
            }
        };

        let jira_api_version = "3".to_string();

        Self {
            api_key: Some(jira_api_key),
            api_version: Some(jira_api_version),
            db_file,
            domain,
            projects: data.projects,
            tickets: data.tickets,
            user_email: jira_user_email,
        }
    }
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Config::default())
    }
}
