use std::collections::HashMap;

use crate::{config::KeyConfig, event::key::Key, jira::tickets::{TicketData, LinkInwardOutwardParent}, widgets::commands::CommandText};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Cell, Clear, Row, Table, TableState},
    Frame,
};

use super::{draw_block_style, draw_highlight_style, commands::CommandInfo, Component, EventState};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    OpenBrowser
}

impl Action {
    pub fn to_command_text(self, key: Key) -> CommandText {
        const CMD_GROUP_GENERAL: &str = "-- General --";
        match self {
            Self::OpenBrowser => CommandText::new(format!("Open Ticket in browser [{key}]"), CMD_GROUP_GENERAL),
        }
    }
}

#[derive(Debug)]
pub struct TicketParentWidget {
    jira_domain: String,
    state: TableState,
    parent_ticket: Option<LinkInwardOutwardParent>,
    pub key_mappings: HashMap<Key, Action>,
}

impl TicketParentWidget {
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
        if !focused {
            self.state.select(None)
        }
        if focused && self.selected().is_some() {
            self.state.select(Some(0))
        }

        let mut rows = Vec::new();

        let title = "Parent";
        let header_cells = ["Key", "Summary", "Priority", "Type", "Status"];
        let headers = Row::new(header_cells);
        let ticket_parent = match &ticket.fields.parent {
            // None => return Ok(()),
            None => {
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

                f.render_widget(table, rect);
                return Ok(());
            }
            Some(i) => {
                self.parent_ticket = Some(i.clone());
                i
            },
            // _ => unreachable!("If there is a link it should be present")
        };
        let priority = match &ticket_parent.fields.priority {
            Some(i) => i.name.as_str(),
            _ => "",
        };
        let item = [
            ticket_parent.key.as_str(),
            ticket_parent.fields.summary.as_str(),
            priority,
            ticket.fields.issuetype.name.as_str(),
            ticket.fields.status.name.as_str(),
        ];
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        // let rows = Row::new(cells).height(height as u16);
        rows.push(Row::new(cells).height(height as u16));
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

impl TicketParentWidget {
    pub fn new(key_config: KeyConfig, jira_domain: &str) -> Self {
        let state = TableState::default();

        let key_mappings = {
            let mut map = HashMap::new();
            map.insert(key_config.open_browser, Action::OpenBrowser);
            map
        };
        Self {
            jira_domain: jira_domain.to_string(),
            key_mappings,
            state,
            parent_ticket: None
        }
    }

    pub fn selected(&self) -> Option<LinkInwardOutwardParent> {
        if self.parent_ticket.is_some() {
            return self.parent_ticket.clone()
        }

        None
    }

    pub fn open_browser(&mut self) {
        if self.selected().is_some() {
            let parent_details  = self.selected().unwrap();
            let url = self.jira_domain.clone() + "/browse/" + &parent_details.key;
            match open::that(url.clone()) {
                Ok(()) => {}
                Err(e) => {
                    // todo!("Add error condition");
                    panic!("{:?} url: {:?}", e, url);
                }
            }
        }
    }
}

impl Component for TicketParentWidget {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Some(action) = self.key_mappings.get(&key) {
            use Action::*;
            match *action {
                OpenBrowser => self.open_browser(),
            }
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}
