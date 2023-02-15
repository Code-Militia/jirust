use surrealdb::Surreal;
use surrealdb::Error as SurrealDbError;
use surrealdb::engine::any::Any;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{config::KeyConfig, event::key::Key, jira::{issue::{TicketData, JiraTickets}, auth::JiraAuth}};

use super::{commands::CommandInfo, Component, EventState, StatefulDrawableComponent};

type SurrealAny = Surreal<Any>;

pub struct TicketComponent {
    key_config: KeyConfig,
    state: ListState,
    ticket: Vec<TicketData>,
}

impl TicketComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = ListState::default();

        return Self {
            state,
            ticket: vec![],
            key_config,
        };
    }

    pub fn next_ticket(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + line >= self.ticket.len() {
                    Some(self.ticket.len() - 1)
                } else {
                    Some(i + line)
                }
            }
            None => None,
        };

        self.state.select(i);
    }

    pub fn previous_ticket(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= line {
                    Some(0)
                } else {
                    Some(i - line)
                }
            }
            None => None,
        };

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.ticket.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.ticket.is_empty() {
            return;
        }
        self.state.select(Some(self.ticket.len() - 1));
    }

    pub fn selected_ticket(&self) -> Option<&TicketData> {
        match self.state.selected() {
            Some(i) => self.ticket.get(i),
            None => None,
        }
    }

    pub async fn update(
        &self,
        db: &SurrealAny,
        jira_auth: &JiraAuth,
        project_key: &str,
        ticket: &JiraTickets,
        ) -> Result<Vec<TicketData>, SurrealDbError> {
        let update_ticket = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
        Ok(update_ticket)
    }
}

impl StatefulDrawableComponent for TicketComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        _rect: Rect,
        _focused: bool,
    ) -> anyhow::Result<()> {
        let width = 80;
        let height = 20;
        let isus = &self.ticket;
        let mut tickets: Vec<ListItem> = Vec::new();
        for i in isus {
            tickets.push(ListItem::new(vec![Spans::from(Span::raw(&i.key))]).style(Style::default()))
        }

        let ticket_block = List::new(tickets)
            .block(Block::default().borders(Borders::ALL).title("Issues"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(Style::default());

        let area = Rect::new(
            (f.size().width.saturating_sub(width)) / 2,
            (f.size().height.saturating_sub(height)) / 2,
            width.min(f.size().width),
            height.min(f.size().height),
        );

        f.render_widget(Clear, area);
        f.render_stateful_widget(ticket_block, area, &mut self.state);

        Ok(())
    }
}

impl Component for TicketComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_ticket(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous_ticket(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_ticket(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_ticket(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.go_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.go_to_top();
            return Ok(EventState::Consumed);
        }
        return Ok(EventState::NotConsumed);
    }
}
