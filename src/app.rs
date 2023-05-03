use crate::jira::tickets::{PostTicketTransition, TicketTransition};
use crate::widgets::comments::{CommentContents, CommentsList};
use crate::widgets::comments_add::CommentAdd;
use crate::widgets::components::ComponentsWidget;
use crate::widgets::description::DescriptionWidget;
use crate::widgets::error::ErrorComponent;
use crate::widgets::labels::LabelsWidget;
use crate::widgets::parent::TicketParentWidget;
use crate::widgets::search_projects::SearchProjectsWidget;
use crate::widgets::search_tickets::SearchTicketsWidget;
use crate::widgets::ticket_relation::RelationWidget;
use crate::widgets::ticket_transition::TransitionWidget;
use crate::widgets::tickets::TicketWidget;
use crate::widgets::{DrawableComponent, InputMode};
use crate::{
    config::Config,
    event::key::Key,
    widgets::{Component, EventState},
};
use crate::{jira::Jira, widgets::projects::ProjectsWidget};
use tui::layout::Rect;
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
    CommentsList,
    CommentView,
    CommentsAdd,
    Components,
    Description,
    Labels,
    Projects,
    SearchProjects,
    SearchTickets,
    Tickets,
    TicketRelation,
    TicketTransition,
}

pub struct App {
    comments_list: CommentsList,
    comment_contents: CommentContents,
    comment_add: CommentAdd,
    components: ComponentsWidget,
    description: DescriptionWidget,
    focus: Focus,
    jira: Jira,
    labels: LabelsWidget,
    // load_state: LoadState,
    parent: TicketParentWidget,
    projects: ProjectsWidget,
    relation: RelationWidget,
    search_projects: SearchProjectsWidget,
    search_tickets: SearchTicketsWidget,
    tickets: TicketWidget,
    ticket_transition: TransitionWidget,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub async fn new(config: Config) -> anyhow::Result<App> {
        let mut jira = Jira::new(
            &config.jira_config.domain,
            &config.jira_config.api_key,
            &config.jira_config.api_version,
            &config.jira_config.user_email,
        )
        .await?;
        let projects = &jira.get_jira_projects().await?;

        Ok(Self {
            comments_list: CommentsList::new(config.key_config.clone()),
            comment_contents: CommentContents::new(config.key_config.clone()),
            comment_add: CommentAdd::new(),
            components: ComponentsWidget::new(config.key_config.clone()),
            config: config.clone(),
            description: DescriptionWidget::new(config.key_config.clone()),
            error: ErrorComponent::new(config.key_config.clone()),
            focus: Focus::Projects,
            jira,
            labels: LabelsWidget::new(config.key_config.clone()),
            // load_state: LoadState::Complete,
            parent: TicketParentWidget::new(),
            projects: ProjectsWidget::new(projects, config.key_config.clone(), config.jira_config.domain.clone()),
            relation: RelationWidget::new(config.key_config.clone()),
            search_projects: SearchProjectsWidget::new(projects),
            search_tickets: SearchTicketsWidget::new(),
            tickets: TicketWidget::new(config.key_config.clone(), config.jira_config.domain.clone()),
            ticket_transition: TransitionWidget::new(Vec::new(), config.key_config.clone()),
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::Projects = self.focus {
            self.projects
                .draw(f, matches!(self.focus, Focus::Projects), f.size())?;
            self.error.draw(f, Rect::default(), false)?;

            return Ok(());
        }

        if let Focus::SearchProjects = self.focus {
            self.search_projects.draw(f)?;
            self.error.draw(f, Rect::default(), false)?;
            return Ok(());
        }

        if let Focus::SearchTickets = self.focus {
            self.search_tickets.draw(f)?;
            self.error.draw(f, Rect::default(), false)?;
            return Ok(());
        }

        if let Focus::TicketTransition = self.focus {
            self.ticket_transition.draw(
                f,
                matches!(self.focus, Focus::TicketTransition),
                f.size(),
            )?;
        }

        if let Focus::CommentsAdd = self.focus {
            self.comment_add.draw(f)?;
            return Ok(());
        }

        if let Focus::TicketTransition = self.focus {
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

        if let Focus::CommentsList = self.focus {
            self.comments_list
                .draw(f, matches!(self.focus, Focus::Projects), f.size())?;
            return Ok(());
        }

        if let Focus::CommentView = self.focus {
            self.comment_contents.draw(
                f,
                self.comments_list.selected(),
                matches!(self.focus, Focus::CommentView),
            )?;
            return Ok(());
        }

        self.error.draw(f, Rect::default(), false)?;

        Ok(())
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.component_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if self.move_focus(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        Ok(EventState::NotConsumed)
    }

    pub async fn next_project_page(&mut self) -> anyhow::Result<()> {
        self.jira.get_next_project_page().await?;
        Ok(())
    }

    pub async fn previous_project_page(&mut self) -> anyhow::Result<()> {
        self.jira.get_projects_previous_page().await?;
        Ok(())
    }

    pub async fn update_projects(&mut self) -> anyhow::Result<()> {
        self.jira.get_jira_projects().await?;
        self.projects.update(&self.jira.projects);
        Ok(())
    }

    pub async fn next_ticket_page(&mut self) -> anyhow::Result<()> {
        let project = self.projects.selected().unwrap();
        self.jira.get_next_ticket_page(&project.key).await?;
        self.tickets.update(&self.jira.tickets.issues).await?;
        Ok(())
    }

    pub async fn previous_ticket_page(&mut self) -> anyhow::Result<()> {
        let project = self.projects.selected().unwrap();
        self.jira.get_previous_tickets_page(&project.key).await?;
        self.tickets.update(&self.jira.tickets.issues).await?;
        Ok(())
    }

    pub async fn update_tickets(&mut self) -> anyhow::Result<()> {
        let project = self.projects.selected().unwrap();
        self.jira.get_jira_tickets(&project.key).await?;
        self.tickets.update(&self.jira.tickets.issues).await?;
        Ok(())
    }

    // pub async fn update_tickets_from_projects_cache(&mut self) -> anyhow::Result<()> {
    //     match self.search_projects.selected() {
    //         Some(project) => {
    //             self.jira.get_jira_tickets(&project).await?;
    //             self.tickets.update(&self.jira.tickets.issues).await?;
    //         }
    //         None => {}
    //     };
    //     Ok(())
    // }

    pub async fn update_search_tickets(&mut self) -> anyhow::Result<()> {
        self.search_tickets.update(self.tickets.tickets.clone());
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

    pub async fn update_comments_view(&mut self) -> anyhow::Result<()> {
        let comments = match self.tickets.selected() {
            None => return Ok(()),
            Some(t) => t.get_comments(&self.jira.db, &self.jira.client).await?,
        };
        self.comments_list.comments = Some(comments);
        Ok(())
    }

    pub async fn add_jira_comment(&mut self) -> anyhow::Result<()> {
        let comment = &self.comment_add.messages;
        let ticket = match self.tickets.selected() {
            None => return Ok(()),
            Some(t) => t,
        };
        if comment.is_empty() && self.comment_add.push_comment {
            let comment = comment.join(" \n ");
            ticket
                .add_comment(&self.jira.db, &comment, &self.jira.client)
                .await?;
            self.comment_add.messages.clear();
            self.comment_add.push_comment = false;
            return Ok(());
        };
        Ok(())
    }

    pub async fn update_ticket_transitions(&mut self) -> anyhow::Result<()> {
        let ticket = match self.tickets.selected() {
            None => return Ok(()),
            Some(t) => t,
        };

        let transitions = ticket.get_transitions(&self.jira.client).await?;
        self.ticket_transition.update(&transitions);
        Ok(())
    }

    pub async fn move_ticket(&mut self) -> anyhow::Result<()> {
        if self.ticket_transition.push_transition {
            let ticket = match self.tickets.selected() {
                None => return Ok(()),
                Some(t) => t,
            };
            let transition = self.ticket_transition.selected_transition().unwrap();
            let data = PostTicketTransition {
                transition: TicketTransition {
                    id: transition.id.clone(),
                    name: transition.name.clone(),
                },
            };
            ticket.transition(data, &self.jira.client).await?;
            self.focus = Focus::Tickets;
            self.ticket_transition.push_transition = false;
            let project = self.projects.selected().unwrap();
            self.jira.get_and_record_tickets(&project.key).await?;
        }
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
            Focus::CommentsList => {
                if self.comments_list.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::CommentView => {
                if self.comment_contents.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::CommentsAdd => {
                if self.comment_add.event(key)?.is_consumed() {
                    self.add_jira_comment().await?;
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
            }
            Focus::Projects => {
                if self.projects.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::SearchProjects => {
                if self.search_projects.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            Focus::SearchTickets => {
                if self.search_tickets.event(key)?.is_consumed() {
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
            }
            Focus::TicketTransition => {
                if self.ticket_transition.event(key)?.is_consumed() {
                    self.move_ticket().await?;
                    self.update_tickets().await?;
                    return Ok(EventState::Consumed);
                }
            }
        }

        Ok(EventState::NotConsumed)
    }

    async fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.config.key_config.focus_projects {
            self.focus = Focus::Projects;
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::CommentsList => {
                if key == self.config.key_config.esc {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
                    self.focus = Focus::CommentView;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::CommentView => {
                if key == self.config.key_config.esc {
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::CommentsAdd => {
                if key == self.config.key_config.esc {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Components => {
                if key == self.config.key_config.focus_above {
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_below {
                    self.focus = Focus::TicketRelation;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.update_comments_view().await?;
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Description => {
                if key == self.config.key_config.focus_left {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.update_comments_view().await?;
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Labels => {
                if key == self.config.key_config.focus_above {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
                if key == self.config.key_config.focus_below {
                    self.update_components().await?;
                    self.focus = Focus::Components;
                    return Ok(EventState::Consumed);
                }
                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.update_comments_view().await?;
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Projects => {
                if key == self.config.key_config.enter {
                    self.update_tickets().await?;
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.search_cache {
                    self.focus = Focus::SearchProjects;
                    self.search_projects.input_mode = InputMode::Editing;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.next_page {
                    self.next_project_page().await?;
                    self.update_projects().await?;
                    self.focus = Focus::Projects;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.previous_page {
                    self.previous_project_page().await?;
                    self.update_projects().await?;
                    self.focus = Focus::Projects;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::SearchProjects => {
                if key == self.config.key_config.enter {
                    let project_input = &self.search_projects.input;
                    if self.search_projects.selected().is_some() {
                        let project = self.search_projects.selected().unwrap();
                        if self.projects.select_project(project).is_ok() {
                            self.update_tickets().await?;
                            self.focus = Focus::Tickets;
                            return Ok(EventState::Consumed);
                        }
                    }
                    if !self.jira.search_jira_projects(project_input).await.is_ok() {
                        if !self.jira.jira_project_api(project_input).await.is_ok() {
                            self.error.set("Unable to locate project in cache and in JIRA \n You may not have access to view project".to_string())?;
                            return Ok(EventState::NotConsumed);
                        }
                    }
                    self.projects.select_project(project_input)?;
                    self.update_tickets().await?;
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.esc {
                    self.focus = Focus::Projects;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::SearchTickets => {
                if key == self.config.key_config.enter {
                    if self.search_tickets.selected().is_some() {
                        let ticket = self.search_tickets.selected().unwrap();
                        if self.tickets.select_ticket(ticket).is_ok() {
                            self.focus = Focus::Description;
                            return Ok(EventState::Consumed);
                        }
                    }

                    if !self
                        .jira
                        .search_jira_tickets(&self.search_tickets.input)
                        .await
                        .is_ok()
                    {
                        if !self
                            .jira
                            .jira_ticket_api(&self.search_tickets.input)
                            .await
                            .is_ok()
                        {
                            self.error.set("Unable to locate ticket in cache and in JIRA \n You may not have access to view ticket".to_string())?;
                            return Ok(EventState::NotConsumed);
                        }
                        self.update_tickets().await?;
                        self.focus = Focus::Description;
                        return Ok(EventState::Consumed);
                    }

                    if self
                        .tickets
                        .select_ticket(&self.search_tickets.input)
                        .is_ok()
                    {
                        self.focus = Focus::Description;
                        return Ok(EventState::Consumed);
                    }
                }
                if key == self.config.key_config.esc {
                    self.focus = Focus::Projects;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::TicketRelation => {
                if key == self.config.key_config.move_up {
                    self.focus = Focus::Components;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.update_comments_view().await?;
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Tickets => {
                if key == self.config.key_config.focus_below {
                    self.update_labels().await?; // Get the current selected ticket and send it
                                                 // to update labels widget
                    self.focus = Focus::Labels;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_right {
                    self.focus = Focus::Description;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_comments {
                    self.update_comments_view().await?;
                    self.focus = Focus::CommentsList;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.focus_add_comments {
                    self.focus = Focus::CommentsAdd;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.next_page {
                    self.next_ticket_page().await?;
                    self.update_tickets().await?;
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.previous_page {
                    self.previous_ticket_page().await?;
                    self.update_tickets().await?;
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.ticket_status {
                    self.update_ticket_transitions().await?;
                    self.focus = Focus::TicketTransition;
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.search_cache {
                    self.focus = Focus::SearchTickets;
                    self.search_tickets.input_mode = InputMode::Editing;
                    self.update_search_tickets().await?;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::TicketTransition => {
                if key == self.config.key_config.esc {
                    self.focus = Focus::Tickets;
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}
