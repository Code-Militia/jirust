use log::{debug, trace};
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
    jira::tickets::{CustomFieldAllowedValues, TicketTransition, TicketTransitions},
};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

#[derive(Debug)]
pub struct TransitionWidget {
    draw_list_float_screen: Option<bool>,
    float_screen_list_state: ListState,
    float_screen_list: Option<Vec<CustomFieldAllowedValues>>,
    focus_float_screen: Option<bool>,
    key_config: KeyConfig,
    state: ListState,
    pub push_transition: bool,
    pub push_transition_reason: Option<String>,
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
            .block(draw_block_style(focused, title))
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

        trace!(
            "Switch to draw float screen is {:?}",
            self.draw_list_float_screen
        );
        if self.draw_list_float_screen == Some(true) {
            let width = 80;
            let height = 20;
            let area = Rect::new(
                (f.size().width.saturating_sub(width)) / 2,
                (f.size().height.saturating_sub(height)) / 2,
                width.min(f.size().width),
                height.min(f.size().height),
            );
            let transition = self.selected_transition().unwrap();
            debug!("{:?}", transition);
            let fields = &transition.fields.as_ref().unwrap().values;
            let mut allowed_values: Vec<CustomFieldAllowedValues> = Vec::new();
            for f in &*fields {
                let field = fields.get(&f.0.to_string()).unwrap();
                if !field.schema.custom.ends_with(":select") {
                    continue;
                }
                allowed_values = field.allowed_values.as_ref().unwrap().to_vec();
            }
            self.float_screen_list = Some(allowed_values.clone());
            return self.draw_select_screen(f, area);
        }

        Ok(())
    }

    fn draw_select_screen<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> anyhow::Result<()> {
        self.focus_float_screen = Some(true);
        let title = "Select Transition Reason";
        let mut list_items: Vec<ListItem> = Vec::new();
        let select_list = self.float_screen_list.clone().unwrap();
        for allowed_value in select_list {
            let value = allowed_value.value;
            list_items
                .push(ListItem::new(vec![Spans::from(Span::raw(value))]).style(Style::default()))
        }
        let list = List::new(list_items)
            .block(draw_block_style(true, title))
            .highlight_style(draw_highlight_style());

        f.render_widget(Clear, rect);
        f.render_stateful_widget(list, rect, &mut self.float_screen_list_state);

        Ok(())
    }
}

impl TransitionWidget {
    pub fn new(transitions: Vec<TicketTransition>, key_config: KeyConfig) -> Self {
        let mut float_screen_list_state = ListState::default();
        float_screen_list_state.select(Some(0));
        let mut state = ListState::default();
        if !transitions.is_empty() {
            state.select(Some(0));
        }
        Self {
            draw_list_float_screen: None,
            float_screen_list_state,
            float_screen_list: None,
            focus_float_screen: None,
            key_config,
            push_transition: false,
            push_transition_reason: None,
            state,
            transitions: Vec::new(),
        }
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

    pub fn check_transition_floating_screen(&mut self) -> bool {
        match self.selected_transition() {
            None => false,
            Some(t) => {
                if t.has_screen.unwrap_or_else(|| false) {
                    self.draw_list_float_screen = Some(true);
                    return true;
                }
                false
            }
        }
    }
}

impl TransitionWidget {
    pub fn next_reason(&mut self, line: usize) {
        let i = match self.float_screen_list_state.selected() {
            Some(i) if i + line >= self.float_screen_list.clone().unwrap().len() => {
                Some(self.float_screen_list.clone().unwrap().len() - 1)
            }
            Some(i) => Some(i + line),
            None => None,
        };

        self.float_screen_list_state.select(i);
    }

    pub fn previous_reason(&mut self, line: usize) {
        let i = match self.float_screen_list_state.selected() {
            Some(i) if i <= line => Some(0),
            Some(i) => Some(i - line),
            None => None,
        };

        self.float_screen_list_state.select(i);
    }

    pub fn go_to_top_reason(&mut self) {
        if self.float_screen_list.clone().unwrap().is_empty() {
            return;
        }
        self.float_screen_list_state.select(Some(0));
    }

    pub fn go_to_bottom_reason(&mut self) {
        if self.float_screen_list.clone().unwrap().is_empty() {
            return;
        }
        self.float_screen_list_state
            .select(Some(self.transitions.len() - 1));
    }

    pub fn selected_transition_reason(&self) -> Option<&CustomFieldAllowedValues> {
        match self.float_screen_list_state.selected() {
            Some(i) => self.float_screen_list.as_ref().unwrap().get(i),
            None => None,
        }
    }
}

impl TransitionWidget {
    fn float_screen_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_reason(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous_reason(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_reason(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_reason(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.go_to_bottom_reason();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.go_to_top_reason();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.esc {
            self.draw_list_float_screen = Some(false);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.enter {
            // TODO: Add comment push based on transition selection
            match self.selected_transition_reason() {
                Some(i) => self.push_transition_reason = Some(i.value.clone()),
                None => {}
            }
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

impl Component for TransitionWidget {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.draw_list_float_screen == Some(true) {
            return self.float_screen_event(key);
        }
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
        } else if key == self.key_config.enter {
            if !self.check_transition_floating_screen() {
                self.push_transition = true;
            }
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}
