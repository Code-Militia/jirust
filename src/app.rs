use crate::components::issues::TicketComponent;
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
    Tickets,
}

pub struct App {
    focus: Focus,
    jira: Jira,
    projects: ProjectsComponent,
    tickets: TicketComponent,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        let jira = Jira::new().await?;
        let projects = &jira.projects.values.clone();

        Ok(Self {
            config: config.clone(),
            error: ErrorComponent::new(config.key_config.clone()),
            focus: Focus::Projects,
            jira,
            tickets: TicketComponent::new(config.key_config.clone()),
            projects: ProjectsComponent::new(projects, config.key_config.clone()),
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

        let ticket_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40)])
            .split(main_chunks[0]);

        let ticket_list = ticket_left_chunks[0];
        let ticket_metadata = ticket_left_chunks[1];

        let ticket_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65)])
            .split(main_chunks[1]);

        let ticket_description = ticket_left_chunks[0];
        let ticket_updates = ticket_left_chunks[1];

        Ok(())
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.component_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
        // todo!("This needs to be filled");
    }

    pub async fn update_tickets(&self, project_key: &str) -> anyhow::Result<()> {
        // if let Some(project) = self.projects.selected_project()
        self.tickets.update(&self.jira.db, &self.jira.auth, project_key, &self.jira.tickets).await?;
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

                if let Some(project) = self.projects.selected_project() {
                    if key == self.config.key_config.enter {
                        self.update_tickets(&project.key).await?;
                        return Ok(EventState::Consumed);
                    }
                }
            }
            Focus::Tickets => {
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
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Tickets => {
                todo!("Add keys for issues");
                // if key == self.config.key_config.enter {
                //
                // }
            }
        }
        return Ok(EventState::NotConsumed);
    }
}
