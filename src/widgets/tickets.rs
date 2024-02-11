use crate::{config::KeyConfig, events::key::Key, jira::tickets::TicketData};
use std::collections::HashMap;

use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::{
    commands::{CommandInfo, CommandText},
    draw_block_style, draw_highlight_style, Component, EventState,
};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    OpenBrowser,
    Next(usize),
    Previous(usize),
    Last,
    First,
    ScrollDownDescription(u16),
    ScrollUpDescription(u16),
}

impl Action {
    pub fn to_command_text(self, key: Key) -> CommandText {
        const CMD_GROUP_GENERAL: &str = "-- General --";
        match self {
            Self::OpenBrowser => {
                CommandText::new(format!("Open Ticket in browser [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::Next(line) => CommandText::new(format!("Next {line} [{key}]"), CMD_GROUP_GENERAL),
            Self::Previous(line) => {
                CommandText::new(format!("Previous {line} [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::Last => CommandText::new(format!("Last [{key}]"), CMD_GROUP_GENERAL),
            Self::First => CommandText::new(format!("First [{key}]"), CMD_GROUP_GENERAL),
            Self::ScrollDownDescription(line) => CommandText::new(
                format!("Scroll down description {line} [{key}]"),
                CMD_GROUP_GENERAL,
            ),
            Self::ScrollUpDescription(line) => CommandText::new(
                format!("Scroll up description {line} [{key}]"),
                CMD_GROUP_GENERAL,
            ),
        }
    }
}

#[derive(Debug)]
pub struct TicketWidget {
    jira_domain: String,
    state: TableState,
    scroll: u16,
    ticket_description: Option<String>,
    pub tickets: Vec<TicketData>,
    pub key_mappings: HashMap<Key, Action>,
}

impl TicketWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        description_frame: Rect,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let title = "Tickets";

        let header_cells = [
            "Key", "Priority", "Type", "Status", "Assignee", "Creator", "Reporter",
        ];
        let headers = Row::new(header_cells);
        let tickets = self.tickets.clone();
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

        match self.selected() {
            Some(ticket) => {
                let summary = format!("{:} - {:}", ticket.key, ticket.fields.summary.clone());
                let description = ticket.rendered_fields.description.clone();
                self.draw_description(f, focused, description_frame, summary, description)
            }
            None => {
                self.draw_description(f, focused, description_frame, String::new(), String::new())
            }
        }?;

        f.render_widget(Clear, rect);
        f.render_stateful_widget(table, rect, &mut self.state);

        Ok(())
    }

    fn draw_description<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
        summary: String,
        description: String,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let summary_title = "Summary";
        let title = "Description";

        let text = match &self.ticket_description {
            Some(d) => d.clone(),
            None => {
                self.ticket_description = Some(parse_html(&description));
                parse_html(&description)
            }
        };

        let summary_paragraph = Paragraph::new(summary)
            .block(draw_block_style(focused, summary_title))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        let paragraph = Paragraph::new(text)
            .block(draw_block_style(focused, title))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Percentage(85)])
            .split(rect);

        f.render_widget(summary_paragraph, main_chunks[0]);
        f.render_widget(paragraph, main_chunks[1]);

        Ok(())
    }
}

impl TicketWidget {
    pub fn new(key_config: KeyConfig, jira_domain: String) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        let key_mappings = {
            let mut map = HashMap::new();
            map.insert(key_config.open_browser, Action::OpenBrowser);
            map.insert(key_config.scroll_down, Action::Next(1));
            map.insert(key_config.scroll_up, Action::Previous(1));
            map.insert(key_config.scroll_down_multiple_lines, Action::Next(10));
            map.insert(key_config.scroll_up_multiple_lines, Action::Previous(10));
            map.insert(key_config.scroll_to_bottom, Action::Last);
            map.insert(key_config.scroll_to_top, Action::First);
            map.insert(key_config.page_down, Action::ScrollDownDescription(1));
            map.insert(key_config.page_up, Action::ScrollUpDescription(1));
            map
        };

        Self {
            jira_domain,
            key_mappings,
            scroll: 0,
            state,
            ticket_description: None,
            tickets: vec![],
        }
    }

    pub fn next(&mut self, line: usize) {
        if self.tickets.is_empty() {
            return;
        }
        self.ticket_description = None;
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.tickets.len() - 1));

        self.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        self.ticket_description = None;
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

        self.select(i);
    }

    pub fn go_to_top(&mut self) {
        self.ticket_description = None;
        if self.tickets.is_empty() {
            return;
        }
        self.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        self.ticket_description = None;
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
        if let Some(ti) = ticket_index {
            self.tickets.swap_remove(ti);
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

    // TODO: This needs to be refactored it always returns an OK
    pub fn select_ticket(&mut self, ticket_key: &str) -> anyhow::Result<()> {
        let ticket_index = self
            .tickets
            .iter()
            .position(|ticket| ticket.key == ticket_key);
        self.select(ticket_index);
        Ok(())
    }

    pub fn scroll_down_description(&mut self, lines: u16) {
        if self.selected().is_some() {
            self.scroll = self.scroll.saturating_add(lines);
            if self.scroll >= 100 {
                self.scroll = 0
            }
        }
    }

    pub fn scroll_up_description(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    pub async fn update(
        &mut self,
        mut tickets: Vec<TicketData>,
        clear: bool,
    ) -> anyhow::Result<()> {
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
        if let Some(action) = self.key_mappings.get(&key) {
            use Action::*;
            match *action {
                OpenBrowser => self.open_browser(),
                Next(line) => self.next(line),
                Previous(line) => self.previous(line),
                Last => self.go_to_bottom(),
                First => self.go_to_top(),
                ScrollDownDescription(line) => self.scroll_down_description(line),
                ScrollUpDescription(line) => self.scroll_up_description(line),
            }
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}
