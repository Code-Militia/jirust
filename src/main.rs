mod jira;
mod jtui;
mod activity_manager;
mod ui;

use jira::auth::{JiraAuth, jira_authentication};
use jira::projects::JiraProjects;
use chrono;
use fern;
use surrealdb::{Datastore, Session};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

pub type DB = (Datastore, Session);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db: &DB = &(
        Datastore::new("memory").await?,
        Session::for_db("jira", "jira"),
    );

    let auth: JiraAuth = jira_authentication();
    
    let projects = JiraProjects { auth: &auth, db_connection: &db };
    projects.save_jira_projects();

    todo!("Start component and mock component for projects");
    todo!("Create UI with grey background, white text, and yellow highlights");
}
