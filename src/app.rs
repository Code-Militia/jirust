use std::sync::Arc;

use surrealdb::{Datastore, Session};
use tui::{layout::{Layout, Direction, Constraint}, Frame, backend::Backend};
use crate::jira::{Jira, projects::JiraProjects};
use crate::jira::auth::JiraAuth;
use crate::components::projects::ProjectsComponent;

pub enum Focus {
    ProjectsList
}

pub struct App {
    auth: Arc<JiraAuth>,
    projects: ProjectsComponent,
    issues: Option<String>,
    db: Arc<DB>,
    focus: Focus
}
pub type DB = (Datastore, Session);

impl App {
    pub async fn new(auth: JiraAuth, db: DB) -> anyhow::Result<App> {
        // Instantiate Jira with Arc
        // If I need to get projects, I would send all of Jira to get projects method
        let jira = Arc::new(Jira::new().await?);
        let jira_projects = JiraProjects::new(&jira);

        Ok(Self {
            auth: Arc::new(auth),
            projects: ProjectsComponent::new(),
            issues: None,
            db: Arc::new(db),
            focus: Focus::ProjectsList
        })
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::ProjectsList = self.focus {
            todo!("Make and draw out projects component"); 
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(100)
                ]
            )
            .split(f.size());

        let issues_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(40),
                ]
            )
            .split(main_chunks[0]);

        let issues_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(65)
                ]
            )
            .split(main_chunks[1]);

        Ok(())
    }
}
