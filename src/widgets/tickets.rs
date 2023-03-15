use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Cell, Clear, ListState, Row, Table, TableState},
    Frame,
};

use crate::{
    config::KeyConfig,
    event::key::Key,
    jira::{
        auth::JiraClient,
        tickets::{JiraTickets, TicketData},
    },
};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct TicketWidget {
    key_config: KeyConfig,
    state: TableState,
    pub tickets: Vec<TicketData>,
}

impl TicketWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let title = "Tickets";

        let header_cells = [
            "Key", "Priority", "Type", "Status", "Assignee", "Creator", "Reporter",
        ];
        let headers = Row::new(header_cells);
        let tickets = &self.tickets;
        let rows = tickets.iter().map(|ticket| {
            let assignee = match &ticket.fields.assignee {
                Some(i) => i.display_name.as_str(),
                _ => "Unassigned",
            };
            let creator = match &ticket.fields.creator {
                Some(i) => i.display_name.as_str(),
                _ => "",
            };
            let reporter = match &ticket.fields.reporter {
                Some(i) => i.display_name.as_str(),
                _ => "",
            };
            let priority = match &ticket.fields.priority {
                Some(i) => i.name.as_str(),
                _ => "",
            };
            let item = [
                ticket.key.as_str(),
                priority,
                ticket.fields.issuetype.name.as_str(),
                ticket.fields.status.name.as_str(),
                assignee,
                creator,
                reporter,
            ];
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(*c));
            Row::new(cells).height(height as u16)
        });
        let table = Table::new(rows)
            .header(headers)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style())
            .widths(&[
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]);

        f.render_widget(Clear, rect);
        f.render_stateful_widget(table, rect, &mut self.state);

        Ok(())
    }
}

impl TicketWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut components_state = ListState::default();
        let mut labels_state = ListState::default();
        let mut state = TableState::default();
        components_state.select(Some(0));
        labels_state.select(Some(0));
        state.select(Some(0));

        return Self {
            key_config,
            tickets: vec![],
            state,
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.tickets.len() - 1));

        self.state.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.state.select(Some(self.tickets.len() - 1));
    }

    pub fn selected(&self) -> Option<&TicketData> {
        match self.state.selected() {
            Some(i) => self.tickets.get(i),
            None => None,
        }
    }

    pub async fn update(
        &mut self,
        db: &SurrealAny,
        jira_auth: &JiraClient,
        project_key: &str,
        ticket: &JiraTickets,
    ) -> anyhow::Result<()> {
        self.tickets = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
        Ok(())
    }
}

impl Component for TicketWidget {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous(10);
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
