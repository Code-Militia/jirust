use crate::{event::key::Key, jira::tickets::TicketComponent};
use log::info;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::EventState;

#[derive(Debug)]
pub struct ComponentsWidget {
    key_config: KeyConfig,
    components: Vec<String>,
    state: ListState,
}

impl ComponentsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let components: Vec<_> = ticket
            .fields
            .components
            .iter()
            .map(|component| ListItem::new(component.name.as_str()))
            .collect();

        let block = List::new(components)
            .block(Block::default().borders(Borders::ALL).title("Components"))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(block, rect, &mut self.state);

        Ok(())
    }
}

impl ComponentsWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        return Self {
            key_config,
            components: vec![],
            state,
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.components.len() - 1));

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
        if self.components.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.components.is_empty() {
            return;
        }
        self.state.select(Some(self.components.len() - 1));
    }

    pub fn selected(&self) -> Option<&String> {
        match self.state.selected() {
            Some(i) => self.components.get(i),
            None => None,
        }
    }

    pub async fn update(&mut self, components: &Vec<TicketComponent>) -> anyhow::Result<()> {
        self.components = components
            .iter()
            .map(|component| component.name.clone())
            .collect();
        Ok(())
    }
}

impl ComponentsWidget {
    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
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
