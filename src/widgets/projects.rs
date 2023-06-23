use std::collections::HashMap;
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
    jira::projects::Project,
};
use crate::widgets::commands::CommandText;

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Down(usize),
    Up(usize),
    Bottom,
    Top,
}

impl Action {
    pub fn to_command_text(self, key: Key) -> CommandText {
        const CMD_GROUP_GENERAL: &str = "-- General --";
        match self {
            Self::Down(line) =>
                CommandText::new(format!("Scroll down {line} [{key}]"), CMD_GROUP_GENERAL),
            Self::Up(line) =>
                CommandText::new(format!("Scroll up {line} [{key}]"), CMD_GROUP_GENERAL),
            Self::Bottom =>
                CommandText::new(format!("Scroll to bottom [{key}]"), CMD_GROUP_GENERAL),
            Self::Top =>
                CommandText::new(format!("Scroll to top [{key}]"), CMD_GROUP_GENERAL),
        }
    }
}

pub struct ProjectsWidget {
    pub projects: Vec<Project>,
    state: ListState,
    pub key_mappings: HashMap<Key, Action>,
}

impl ProjectsWidget {
    pub fn new(projects: &Vec<Project>, key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        if !projects.is_empty() {
            state.select(Some(0));
        }

        let key_mappings = {
            let mut map = HashMap::new();
            map.insert(Key::Down, Action::Down(1));
            map.insert(Key::Up, Action::Up(1));

            map.insert(key_config.scroll_down, Action::Down(1));
            map.insert(key_config.scroll_up, Action::Up(1));
            map.insert(key_config.scroll_down_multiple_lines, Action::Down(10));
            map.insert(key_config.scroll_up_multiple_lines, Action::Up(10));
            map.insert(key_config.scroll_to_bottom, Action::Bottom);
            map.insert(key_config.scroll_to_top, Action::Top);
            map
        };

        Self {
            state,
            projects: projects.to_vec(),
            key_mappings,
        }
    }

    pub fn next(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i + line >= self.projects.len() => Some(self.projects.len() - 1),
            Some(i) => Some(i + line),
            None => None,
        };

        self.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i <= line => Some(0),
            Some(i) => Some(i - line),
            None => None,
        };

        self.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.select(Some(self.projects.len() - 1));
    }

    pub fn selected(&self) -> Option<&Project> {
        match self.state.selected() {
            Some(i) => self.projects.get(i),
            None => None,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        if index.is_some() {
            self.state.select(index)
        }
    }

    pub fn select_project(&mut self, ticket_key: &str) -> anyhow::Result<()> {
        let ticket_index = self
            .projects
            .iter()
            .position(|ticket| ticket.key == ticket_key);
        self.select(ticket_index);
        Ok(())
    }

    pub async fn update(&mut self, jira_projects: &[Project]) -> anyhow::Result<()> {
        self.projects = jira_projects.to_owned();
        Ok(())
    }
}

impl ProjectsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        _rect: Rect,
        // _focused: bool,
    ) -> anyhow::Result<()> {
        let title = "Projects";
        let mut list_items: Vec<ListItem> = Vec::new();
        for p in &self.projects {
            list_items
                .push(ListItem::new(vec![Spans::from(Span::raw(&p.key))]).style(Style::default()))
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

        Ok(())
    }
}

impl Component for ProjectsWidget {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Some(action) = self.key_mappings.get(&key) {
            use Action::*;
            match *action {
                Down(line) => self.next(line),
                Up(line) => self.previous(line),
                Bottom => self.go_to_bottom(),
                Top => self.go_to_top(),
            }
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}
