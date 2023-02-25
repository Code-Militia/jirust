use crate::widgets::components::ComponentsWidget;
use crate::widgets::labels::LabelsWidget;
use crate::widgets::tickets::TicketWidget;
use crate::{
    config::Config,
    event::key::Key,
    widgets::{error::ErrorComponent, Component, EventState, StatefulDrawableComponent},
};
use crate::{jira::Jira, widgets::projects::ProjectsWidget};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

// pub enum LoadState {
//     Complete,
//     Loading,
// }

pub enum Focus {
    Projects,
    Tickets,
    Labels,
    Components,
}

pub struct App {
    components: ComponentsWidget,
    focus: Focus,
    jira: Jira,
    labels: LabelsWidget,
    // load_state: LoadState,
    projects: ProjectsWidget,
    tickets: TicketWidget,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        let jira = Jira::new().await?;
        let projects = &jira.projects.values.clone();

        Ok(Self {
            components: ComponentsWidget::new(config.key_config.clone()),
            config: config.clone(),
            error: ErrorComponent::new(config.key_config.clone()),
            focus: Focus::Projects,
            jira,
            labels: LabelsWidget::new(config.key_config.clone()),
            // load_state: LoadState::Complete,
            tickets: TicketWidget::new(config.key_config.clone()),
            projects: ProjectsWidget::new(projects, config.key_config.clone()),
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::Projects = self.focus {
            self.projects.draw(f, matches!(self.focus, Focus::Projects), f.size())?;

            // TODO: Handle errors and help
            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(f.size());

        let ticket_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(main_chunks[0]);

        let ticket_list = ticket_left_chunks[0];
        let ticket_labels = ticket_left_chunks[1];
        let ticket_component = ticket_left_chunks[2];

        let ticket_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(main_chunks[1]);

        // let ticket_description = ticket_right_chunks[0];
        // let ticket_worklog = ticket_right_chunks[1];

        self.tickets
            .draw(f, matches!(self.focus, Focus::Tickets), ticket_list)?;
        self.labels.draw(
            f,
            matches!(self.focus, Focus::Labels),
            ticket_labels,
            self.tickets.selected(),
        )?;
        // self.tickets.draw_components(f, ticket_component)?;
        self.components.draw(
            f,
            matches!(self.focus, Focus::Components),
            ticket_component,
            self.tickets.selected(),
        )?;
        // self.tickets.draw_description(f, ticket_description)?;
        // self.tickets.draw_work_log(f, ticket_worklog)?;

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
    }

    pub async fn update_tickets(&mut self) -> anyhow::Result<()> {
        let project = self.projects.selected_project().unwrap();
        self.tickets
            .update(
                &self.jira.db,
                &self.jira.auth,
                &project.key,
                &self.jira.tickets,
            )
            .await?;
        self.focus = Focus::Tickets;
        Ok(())
    }

    pub async fn update_labels(&mut self) -> anyhow::Result<()> {
        let empty_vec = Vec::new();
        match self.tickets.selected() {
            None => {
                self.labels.update(&empty_vec).await?;
            }
            Some(t) => {
                self.labels.update(&t.fields.labels).await?;
            }
        };
        Ok(())
    }

    pub async fn update_components(&mut self) -> anyhow::Result<()> {
        let empty_vec = Vec::new();
        match self.tickets.selected() {
            None => {
                self.components.update(&empty_vec).await?;
            }
            Some(t) => {
                self.components.update(&t.fields.components).await?;
            }
        };
        Ok(())
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
                    self.update_tickets().await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Tickets => {
                if self.tickets.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_below {
                    self.update_labels().await?; // Get the current selected ticket and send it
                                                 // to update labels widget
                }
            }
            Focus::Labels => {
                if self.labels.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_below {
                    self.update_components().await?;
                }
            }
            Focus::Components => {
                if self.components.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
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
                if key == self.config.key_config.focus_below {
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Labels => {
                if key == self.config.key_config.focus_above {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
                if key == self.config.key_config.focus_below {
                    self.focus = Focus::Components;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Components => {
                if key == self.config.key_config.focus_above {
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }
            }
        }
        return Ok(EventState::NotConsumed);
    }
}
