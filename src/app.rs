use crate::{components::{StatefulDrawableComponent, EventState, Component}, event::key::Key, config::Config};
use tui::{layout::{Layout, Direction, Constraint}, Frame, backend::Backend};
use crate::{jira::Jira, components::projects::ProjectsComponent};

pub enum Focus {
    ProjectsList
}

pub struct App {
    // auth: Arc<JiraAuth>,
    // projects: ProjectsComponent,
    // issues: Option<String>,
    // db: Arc<DB>,
    focus: Focus,
    projects: ProjectsComponent,
    config: Config
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        // Instantiate Jira with Arc
        // If I need to get projects, I would send all of Jira to get projects method
        let jira = Jira::new().await?;

        Ok(Self {
            // auth: Arc::new(auth),
            projects: ProjectsComponent::new(&jira.projects.values, config.key_config.clone()),
            // issues: None,
            // db: Arc::new(db),
            focus: Focus::ProjectsList,
            config: config.clone()
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::ProjectsList = self.focus {
            self.projects.draw(
                f,
                Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0],
                false,
            )?;

            // TODO: Handle errors and help
            return Ok(())
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

    fn event() -> anyhow::Result<()> {
        todo!("This needs to be filled");
        // return Ok(EventState::Consumed)
    }

    fn component_event(&self, key: Key) -> anyhow::Result<EventState> {
        /* if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        } */

        /* if !matches!(self.focus, Focus::ConnectionList) && self.help.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        } */

        match self.focus {
            Focus::ProjectsList => {
                if self.projects.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed)
                }

                if key == self.config.key_config.enter {
                    todo!("Execute update issues by selecting project first");
                    // return Ok(EventState::Consumed)
                }
            }
        }

        return Ok(EventState::Consumed)
    }
}
