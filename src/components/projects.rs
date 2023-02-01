use tui::{backend::Backend, Frame, layout::Rect, widgets::{ListState, ListItem, List, Block, Borders, Clear}, text::{Spans, Span}, style::{Style, Color}};
use std::io::Error;

use crate::jira::projects::Project;

use super::StatefulDrawableComponent;

pub struct ProjectsComponent {
    projects: Vec<Project>,
    state: ListState,
}

impl ProjectsComponent {
    pub fn new(projects: Vec<Project>) -> Self {
        let mut state = ListState::default();
        if projects.is_empty() {
            state.select(Some(0));
        }

        return Self {
            state,
            projects
        }
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
            None => None
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
            None => None
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

    pub fn selected_project(&self) -> Option<&Project> {
        match self.state.selected() {
            Some(i) => self.projects.get(i),
            None => None
        }
    }
}

impl StatefulDrawableComponent for ProjectsComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _rect: Rect, _focused: bool) -> Result<(), Error> {
        let width = 80;
        let height = 20;
        let prjs = &self.projects;
        let mut projects: Vec<ListItem> = Vec::new();
        for p in prjs {
            projects.push(
                ListItem::new(vec![Spans::from(Span::raw(p.key))])
                    .style(Style::default()),
            )
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
