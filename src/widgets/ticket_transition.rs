use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    text::{Span, Spans},
    widgets::{Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    config::KeyConfig,
    event::key::Key,
    jira::tickets::{TicketTransition, TicketTransitions},
};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

#[derive(Debug)]
pub struct TransitionWidget {
    key_config: KeyConfig,
    state: ListState,
    pub push_transition: bool,
    pub transitions: Vec<TicketTransition>,
}

impl TransitionWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let title = "Transition";
        let mut list_items: Vec<ListItem> = Vec::new();
        for c in &self.transitions {
            let name = c.name.as_ref().unwrap();
            list_items
                .push(ListItem::new(vec![Spans::from(Span::raw(name))]).style(Style::default()))
        }

        let list = List::new(list_items)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style());

        let width = 80;
        let height = 20;
        let area = Rect::new(
            (f.size().width.saturating_sub(width)) / 2,
            (f.size().height.saturating_sub(height)) / 2,
            width.min(f.size().width),
            height.min(f.size().height),
        );

        f.render_widget(Clear, area);
        f.render_stateful_widget(list, area, &mut self.state);

        Ok(())
    }
}

impl TransitionWidget {
    pub fn new(transitions: Vec<TicketTransition>, key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        if !transitions.is_empty() {
            state.select(Some(0));
        }
        return Self {
            state,
            key_config,
            push_transition: false,
            transitions: Vec::new(),
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i + line >= self.transitions.len() => Some(self.transitions.len() - 1),
            Some(i) => Some(i + line),
            None => None,
        };

        self.state.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i <= line => Some(0),
            Some(i) => Some(i - line),
            None => None,
        };

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.transitions.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.transitions.is_empty() {
            return;
        }
        self.state.select(Some(self.transitions.len() - 1));
    }

    pub fn selected_transition(&self) -> Option<&TicketTransition> {
        match self.state.selected() {
            Some(i) => self.transitions.get(i),
            None => None,
        }
    }

    pub fn update(&mut self, transitions: &TicketTransitions) {
        self.transitions = transitions.transitions.clone();
        if !transitions.transitions.is_empty() {
            self.state.select(Some(0));
        }
    }
}

impl Component for TransitionWidget {
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
        } else if key == self.key_config.move_ticket {
            self.push_transition = true;
            return Ok(EventState::Consumed);
        }
        return Ok(EventState::NotConsumed);
    }
}
