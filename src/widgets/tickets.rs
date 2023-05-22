use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Cell, Clear, ListState, Row, Table, TableState},
    Frame,
};

use crate::{config::KeyConfig, event::key::Key, jira::tickets::TicketData};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

#[derive(Debug)]
pub struct TicketWidget {
    jira_domain: String,
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
            .block(draw_block_style(focused, title))
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
    pub fn new(key_config: KeyConfig, jira_domain: String) -> Self {
        let mut components_state = ListState::default();
        let mut labels_state = ListState::default();
        let mut state = TableState::default();
        components_state.select(Some(0));
        labels_state.select(Some(0));
        state.select(Some(0));

        Self {
            jira_domain,
            key_config,
            tickets: vec![],
            state,
        }
    }

    pub fn next(&mut self, line: usize) {
        if self.tickets.is_empty() {
            return;
        }
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.tickets.len() - 1));

        self.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

        self.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.select(Some(self.tickets.len() - 1))
    }

    pub fn open_browser(&mut self) {
        if self.selected().is_some() {
            let ticket = self.selected().unwrap().clone();
            let url = self.jira_domain.clone() + "/browse/" + &ticket.key;
            match open::that(url.clone()) {
                Ok(()) => {}
                Err(e) => {
                    // todo!("Add error condition");
                    panic!("{:?} url: {:?}", e, url);
                }
            }
        }
    }

    pub fn remove_ticket(&mut self, ticket_key: &str) -> anyhow::Result<()> {
        let ticket_index = self
            .tickets
            .iter()
            .position(|ticket| ticket.key == ticket_key);
        match ticket_index {
            Some(ti) => {
                self.tickets.swap_remove(ti);
            }
            None => {}
        }
        Ok(())
    }

    pub fn selected(&mut self) -> Option<&TicketData> {
        match self.state.selected() {
            Some(i) => {
                let get = self.tickets.get(i);
                get
            }
            None => None,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        if index.is_some() {
            self.state.select(index)
        }
    }

    pub fn select_ticket(&mut self, ticket_key: &str) -> anyhow::Result<()> {
        let ticket_index = self
            .tickets
            .iter()
            .position(|ticket| ticket.key == ticket_key);
        self.select(ticket_index);
        Ok(())
    }

    pub async fn update(&mut self, mut tickets: Vec<TicketData>, clear: bool) -> anyhow::Result<()> {
        if clear {
            self.tickets.clear();
        }
        self.tickets.append(&mut tickets);
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
        } else if key == self.key_config.open_browser {
            self.open_browser();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}
