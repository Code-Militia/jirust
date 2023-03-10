use crate::jira::tickets::CommentBody;
use crate::widgets::comments::CommentsWidget;
use crate::widgets::components::ComponentsWidget;
use crate::widgets::description::DescriptionWidget;
use crate::widgets::labels::LabelsWidget;
use crate::widgets::parent::TicketParentWidget;
use crate::widgets::ticket_relation::RelationWidget;
use crate::widgets::tickets::TicketWidget;
use crate::{
    config::Config,
    event::key::Key,
    widgets::{error::ErrorComponent, Component, EventState},
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
    Comments,
    Components,
    Description,
    Labels,
    Projects,
    Tickets,
    TicketRelation,
}

pub struct App {
    comments: CommentsWidget,
    components: ComponentsWidget,
    description: DescriptionWidget,
    focus: Focus,
    jira: Jira,
    labels: LabelsWidget,
    // load_state: LoadState,
    parent: TicketParentWidget,
    projects: ProjectsWidget,
    relation: RelationWidget,
    tickets: TicketWidget,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        let jira = Jira::new().await?;
        let projects = &jira.projects.values.clone();

        Ok(Self {
            comments: CommentsWidget::new(config.key_config.clone()),
            components: ComponentsWidget::new(config.key_config.clone()),
            config: config.clone(),
            description: DescriptionWidget::new(config.key_config.clone()),
            error: ErrorComponent::new(config.key_config.clone()),
            focus: Focus::Projects,
            jira,
            labels: LabelsWidget::new(config.key_config.clone()),
            // load_state: LoadState::Complete,
            parent: TicketParentWidget::new(),
            projects: ProjectsWidget::new(projects, config.key_config.clone()),
            relation: RelationWidget::new(config.key_config.clone()),
            tickets: TicketWidget::new(config.key_config.clone()),
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::Projects = self.focus {
            self.projects
                .draw(f, matches!(self.focus, Focus::Projects), f.size())?;

            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(f.size());

        let description_metadata = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[0]);

        let ticket_relation = main_chunks[1];

        let ticket_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(40)])
            .split(description_metadata[0]);

        let ticket_metadata_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .split(ticket_left_chunks[1]);

        let ticket_list = ticket_left_chunks[0];
        let ticket_labels = ticket_metadata_chunks[0];
        let ticket_component = ticket_metadata_chunks[1];
        let ticket_parent = ticket_metadata_chunks[2];

        let ticket_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(description_metadata[1]);

        let ticket_description = ticket_right_chunks[0];

        self.tickets
            .draw(f, matches!(self.focus, Focus::Tickets), ticket_list)?;

        self.labels.draw(
            f,
            matches!(self.focus, Focus::Labels),
            ticket_labels,
            self.tickets.selected(),
        )?;

        self.components.draw(
            f,
            matches!(self.focus, Focus::Components),
            ticket_component,
            self.tickets.selected(),
        )?;

        self.description.draw(
            f,
            matches!(self.focus, Focus::Description),
            ticket_description,
            self.tickets.selected(),
        )?;

        self.parent
            .draw(f, false, ticket_parent, self.tickets.selected())?;

        self.relation.draw(
            f,
            matches!(self.focus, Focus::TicketRelation),
            ticket_relation,
            self.tickets.selected(),
        )?;

        if let Focus::Comments = self.focus {
            self.comments.draw(
                f,
                matches!(self.focus, Focus::Projects),
                f.size(),
                self.tickets.selected(),
            )?;
            return Ok(());
        }

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
                &self.jira.client,
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

    pub async fn update_comments(&mut self) -> anyhow::Result<()> {
        match self.tickets.selected() {
            None => {
                return Ok(())
            }
            Some(t) => {
                t.get_comments(&self.jira.db, &self.jira.client).await?;
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
            Focus::Comments => {
                if self.comments.event(key)?.is_consumed() {
                    self.update_comments().await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Components => {
                if self.components.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Description => {
                if self.description.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
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
            Focus::Projects => {
                if self.projects.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
                    self.update_tickets().await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::TicketRelation => {
                if self.relation.event(key)?.is_consumed() {
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
        }

        return Ok(EventState::NotConsumed);
    }

    fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.config.key_config.focus_projects {
            self.focus = Focus::Projects;
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::Comments => {
                if key == self.config.key_config.exit_popup {
                    self.focus = Focus::Tickets
                }
            }
            Focus::Components => {
                if key == self.config.key_config.focus_above {
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_below {
                    self.focus = Focus::TicketRelation
                }

                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.focus = Focus::Comments;
                }
            }
            Focus::Description => {
                if key == self.config.key_config.focus_left {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.focus = Focus::Comments;
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
                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.focus = Focus::Comments;
                }
            }
            Focus::Projects => {
                if key == self.config.key_config.enter {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::TicketRelation => {
                if key == self.config.key_config.move_up {
                    self.focus = Focus::Components;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.focus = Focus::Comments;
                }
            }
            Focus::Tickets => {
                if key == self.config.key_config.focus_below {
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }
                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.focus = Focus::Comments;
                }
            }
        }
        return Ok(EventState::NotConsumed);
    }
}
