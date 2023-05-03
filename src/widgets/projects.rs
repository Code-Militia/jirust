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
    jira::projects::{JiraProjects, Project},
};

use super::{draw_block_style, draw_highlight_style, Component, EventState};

pub struct ProjectsWidget {
    projects: Vec<Project>,
    state: ListState,
    key_config: KeyConfig,
}

impl ProjectsWidget {
    pub fn new(projects: &Vec<Project>, key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        if !projects.is_empty() {
            state.select(Some(0));
        }

        Self {
            state,
            projects: projects.to_vec(),
            key_config,
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

    pub fn select_project(&mut self, project_key: &str) -> anyhow::Result<()> {
        for (index, project_data) in self.projects.iter().enumerate() {
            if project_data.key == project_key.clone() {
                self.select(Some(index));
                return Ok(());
            }
        }

        Ok(())
    }

    pub fn update(&mut self, jira_projects: &JiraProjects) {
        self.projects = jira_projects.values.clone()
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
    // fn commands(&self, _out: &mut Vec<CommandInfo>) {}

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
        Ok(EventState::NotConsumed)
    }
}
