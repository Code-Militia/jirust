use crate::{event::key::Key, jira::tickets::Components};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Clear, List, ListItem, ListState},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, draw_highlight_style, EventState};

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
        focused: bool,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let title = "Components";
        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let list_items: Vec<_> = ticket
            .fields
            .components
            .iter()
            .map(|component| ListItem::new(component.name.as_str()))
            .collect();

        let list = List::new(list_items)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style());

        f.render_stateful_widget(list, rect, &mut self.state);

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
        if self.components.is_empty() {
            return;
        }
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

    pub async fn update(&mut self, components: &Vec<Components>) -> anyhow::Result<()> {
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
