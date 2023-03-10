use log::info;
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
        tickets::{JiraTickets, Links, TicketData},
    },
};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct RelationWidget {
    key_config: KeyConfig,
    state: TableState,
    pub ticket_links: Vec<Links>,
}

impl RelationWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let title = "Relation";
        let header_cells = ["Relation", "Key", "Summary", "Priority", "Type", "Status"];
        let headers = Row::new(header_cells);
        self.ticket_links = ticket.fields.issuelinks.clone();
        let rows = self.ticket_links.iter().map(|link_details| {
            let link_relation_detail;
            let link_relation = match (&link_details.outward_issue, &link_details.inward_issue) {
                (Some(outward), None) => {
                    link_relation_detail = &link_details.link_type.outward;
                    outward
                }
                (None, Some(inward)) => {
                    link_relation_detail = &link_details.link_type.inward;
                    inward
                }
                _ => unreachable!("If there is a link, this should always return"),
            };
            let item = [
                link_relation_detail,
                link_relation.key.as_str(),
                link_relation.fields.summary.as_str(),
                link_relation.fields.priority.name.as_str(),
                link_relation.fields.issuetype.name.as_str(),
                link_relation.fields.status.name.as_str(),
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

impl RelationWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        return Self {
            key_config,
            ticket_links: vec![],
            state,
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.ticket_links.len() - 1));

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
        if self.ticket_links.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.ticket_links.is_empty() {
            return;
        }
        self.state.select(Some(self.ticket_links.len() - 1));
    }

    pub fn selected(&self) -> Option<&Links> {
        match self.state.selected() {
            Some(i) => self.ticket_links.get(i),
            None => None,
        }
    }

    // pub async fn update(
    //     &mut self,
    //     db: &SurrealAny,
    //     jira_auth: &JiraAuth,
    //     project_key: &str,
    //     ticket: &JiraTickets,
    // ) -> anyhow::Result<()> {
    //     self.tickets = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
    //     Ok(())
    // }
}

impl Component for RelationWidget {
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
