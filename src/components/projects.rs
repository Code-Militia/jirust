use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{config::KeyConfig, event::key::Key, jira::projects::Project};

use super::{commands::CommandInfo, Component, EventState, StatefulDrawableComponent};

pub struct ProjectsComponent {
    projects: Vec<String>,
    state: ListState,
    key_config: KeyConfig,
}

impl ProjectsComponent {
    pub fn new(projects: &Vec<Project>, key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        if projects.is_empty() {
            state.select(Some(0));
        }

        let mut projects_list = vec![];

        for project in projects.iter() {
            projects_list.push(project.name.clone())
        }

        return Self {
            state,
            projects: projects_list,
            key_config,
        };
    }

    pub fn next_project(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + line >= self.projects.len() {
                    Some(self.projects.len() - 1)
                } else {
                    Some(i + line)
                }
            }
            None => None,
        };
    }

    pub fn previous_project(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= line {
                    Some(0)
                } else {
                    Some(i - line)
                }
            }
            None => None,
        };
    }

    pub fn go_to_top(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.state.select(Some(self.projects.len() - 1));
    }

    pub fn selected_project(&self) -> Option<&String> {
        match self.state.selected() {
            Some(i) => self.projects.get(i),
            None => None,
        }
    }
}

impl StatefulDrawableComponent for ProjectsComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        _rect: Rect,
        _focused: bool,
    ) -> anyhow::Result<()> {
        let width = 80;
        let height = 20;
        let prjs = &self.projects;
        let mut projects: Vec<ListItem> = Vec::new();
        for p in prjs {
            projects.push(ListItem::new(vec![Spans::from(Span::raw(p))]).style(Style::default()))
        }

        let projects = List::new(projects)
            .block(Block::default().borders(Borders::ALL).title("Projects"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(Style::default());

        let area = Rect::new(
            (f.size().width.saturating_sub(width)) / 2,
            (f.size().height.saturating_sub(height)) / 2,
            width.min(f.size().width),
            height.min(f.size().height),
        );

        f.render_widget(Clear, area);
        f.render_stateful_widget(projects, area, &mut self.state);

        Ok(())
    }
}

impl Component for ProjectsComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_project(1);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up {
            self.previous_project(1);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_project(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_project(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.go_to_bottom();
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_to_top {
            self.go_to_top();
            return Ok(EventState::NotConsumed);
        }
        return Ok(EventState::Consumed);
    }
}
