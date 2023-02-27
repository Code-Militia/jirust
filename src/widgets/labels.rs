use crate::event::key::Key;
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Clear, List, ListItem, ListState},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, draw_highlight_style, EventState};

#[derive(Debug)]
pub struct LabelsWidget {
    key_config: KeyConfig,
    labels: Vec<String>,
    state: ListState,
}

impl LabelsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let title = "Labels";

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let list_items: Vec<_> = ticket
            .fields
            .labels
            .iter()
            .map(|label| ListItem::new(label.as_str()))
            .collect();

        let labels_list = List::new(list_items)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style());

        f.render_stateful_widget(labels_list, rect, &mut self.state);

        Ok(())
    }
}

impl LabelsWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        return Self {
            key_config,
            labels: vec![],
            state,
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.labels.len() - 1));

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
        if self.labels.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.labels.is_empty() {
            return;
        }
        self.state.select(Some(self.labels.len() - 1));
    }

    pub fn selected(&self) -> Option<&String> {
        match self.state.selected() {
            Some(i) => self.labels.get(i),
            None => None,
        }
    }

    pub async fn update(&mut self, labels: &Vec<String>) -> anyhow::Result<()> {
        self.labels = labels.to_vec();
        Ok(())
    }
}

impl LabelsWidget {
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
