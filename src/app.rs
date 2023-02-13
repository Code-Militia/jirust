use crate::{components::projects::ProjectsComponent, jira::Jira};
use crate::{
    components::{error::ErrorComponent, Component, EventState, StatefulDrawableComponent},
    config::{Config, KeyConfig},
    event::key::Key,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub enum Focus {
    Projects,
    Issues,
}

pub struct App {
    // auth: Arc<JiraAuth>,
    // projects: ProjectsComponent,
    // issues: Option<String>,
    // db: Arc<DB>,
    focus: Focus,
    projects: ProjectsComponent,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        // Instantiate Jira with Arc
        // If I need to get projects, I would send all of Jira to get projects method
        let jira = Jira::new().await?;

        Ok(Self {
            // auth: Arc::new(auth),
            // issues: None,
            // db: Arc::new(db),
            config: config.clone(),
            error: ErrorComponent::new(config.key_config.clone()),
            focus: Focus::Projects,
            projects: ProjectsComponent::new(&jira.projects.values, config.key_config.clone()),
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::Projects = self.focus {
            self.projects.draw(
                f,
                Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0],
                false,
            )?;

            // TODO: Handle errors and help
            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(100)])
            .split(f.size());

        let issues_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40)])
            .split(main_chunks[0]);

        let issues_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65)])
            .split(main_chunks[1]);

        Ok(())
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        // let t1 = self.component_event(key).await?;
        if self.component_event(key).await?.is_consumed() {
            // println!("t1 is --- {t1:?}");
            return Ok(EventState::Consumed);
        }

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
        // todo!("This needs to be filled");
    }

    pub async fn update_issues(&self) -> anyhow::Result<()> {
        todo!("create update issues method to force updates from client");
    }

    pub async fn component_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        // if !matches!(self.focus, Focus::Projects) && self.help.event(key)?.is_consumed() {
        //     return Ok(EventState::Consumed);
        // }

        match self.focus {
            Focus::Projects => {
                if self.projects.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
                    todo!("Execute update issues by selecting project first");
                    // return Ok(EventState::Consumed)
                }
            }
            Focus::Issues => {
                todo!("Need to return an issues list");
            }
        }

        return Ok(EventState::NotConsumed);
    }

    fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.config.key_config.focus_projects {
            self.focus = Focus::Projects;
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::Projects => {
                if key == self.config.key_config.enter {
                    self.focus = Focus::Issues;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Issues => {
                todo!("Add keys for issues");
                // if key == self.config.key_config.enter {
                //
                // }
            }
        }
        return Ok(EventState::NotConsumed);
    }
}
