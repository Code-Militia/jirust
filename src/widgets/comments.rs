use std::collections::HashMap;

use crate::{
    event::key::Key,
    jira::tickets::{CommentBody, Comments},
};
use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::config::KeyConfig;

use super::{
    commands::{CommandInfo, CommandText},
    draw_block_style, draw_highlight_style, Component, EventState,
};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    PageDown(u16),
    PageUp(u16),
    NextComment(usize),
    PreviousComment(usize),
    LastComment,
    FirstComment,
}

impl Action {
    pub fn to_command_text(self, key: Key) -> CommandText {
        const CMD_GROUP_GENERAL: &str = "-- General --";
        match self {
            Self::PageDown(line) => {
                CommandText::new(format!("Scroll down {line} [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::PageUp(line) => {
                CommandText::new(format!("Scroll up {line} [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::NextComment(line) => {
                CommandText::new(format!("Next {line} [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::PreviousComment(line) => {
                CommandText::new(format!("Previous {line} [{key}]"), CMD_GROUP_GENERAL)
            }
            Self::LastComment => CommandText::new(format!("Go to last [{key}]"), CMD_GROUP_GENERAL),
            Self::FirstComment => {
                CommandText::new(format!("Go to first [{key}]"), CMD_GROUP_GENERAL)
            }
        }
    }
}

#[derive(Debug)]
pub struct CommentsList {
    comments_parsed: Option<String>,
    scroll: u16,
    state: TableState,
    pub comments: Option<Comments>,
    pub key_mappings: HashMap<Key, Action>,
}

impl CommentsList {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let chunk_constrains = [Constraint::Length(10), Constraint::Min(1)].as_ref();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(chunk_constrains)
            .split(f.size());
        let title = "Comments";

        let header_cells = ["Author", "Updated Author", "Created by", "Updated by"];
        let headers = Row::new(header_cells);
        let rows = match &self.comments {
            None => return Ok(()),
            Some(c) => c.comments.iter().map(|comment_body| {
                let item = [
                    comment_body.author.display_name.as_str(),
                    comment_body.update_author.display_name.as_str(),
                    comment_body.created.as_str(),
                    comment_body.updated.as_str(),
                ];
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().map(|c| Cell::from(*c));
                Row::new(cells).height(height as u16)
            }),
        };
        let table = Table::new(rows)
            .header(headers)
            .block(draw_block_style(focused, title))
            .highlight_style(draw_highlight_style())
            .widths(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]);

        f.render_stateful_widget(table, chunks[0], &mut self.state);

        if self.selected().is_none() {
            return Ok(());
        }
        let comment = self.selected().unwrap().clone();
        let text = match &self.comments_parsed {
            Some(c) => c.clone(),
            None => {
                self.comments_parsed = Some(parse_html(&comment.rendered_body));
                parse_html(&comment.rendered_body)
            }
        };
        // let text = parse_html(&comment.rendered_body.clone());
        let paragraph = Paragraph::new(Span::styled(text, Style::default()))
            .alignment(Alignment::Left)
            .block(draw_block_style(focused, title))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[1]);

        Ok(())
    }
}

impl CommentsList {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        let key_mappings = {
            let mut map = HashMap::new();
            map.insert(key_config.page_down, Action::PageDown(10));
            map.insert(key_config.page_up, Action::PageUp(10));
            map.insert(key_config.scroll_down, Action::NextComment(1));
            map.insert(key_config.scroll_up, Action::PreviousComment(1));
            map.insert(
                key_config.scroll_down_multiple_lines,
                Action::NextComment(10),
            );
            map.insert(
                key_config.scroll_up_multiple_lines,
                Action::PreviousComment(10),
            );
            map.insert(key_config.scroll_to_bottom, Action::LastComment);
            map.insert(key_config.scroll_to_top, Action::FirstComment);
            map
        };
        Self {
            comments: None,
            comments_parsed: None,
            key_mappings,
            scroll: 0,
            state,
        }
    }

    pub fn next(&mut self, line: usize) {
        self.comments_parsed = None;
        let comments = match &self.comments {
            None => return,
            Some(c) => c,
        };
        if comments.comments.is_empty() {
            return;
        }
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(comments.comments.len() - 1));

        self.state.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        self.comments_parsed = None;
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
        self.comments_parsed = None;
        let comments = match &self.comments {
            None => return,
            Some(c) => c,
        };
        if comments.comments.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        self.comments_parsed = None;
        let comments = match &self.comments {
            None => return,
            Some(c) => c,
        };
        if comments.comments.is_empty() {
            return;
        }
        self.state.select(Some(comments.comments.len() - 1));
    }

    pub fn selected(&self) -> Option<&CommentBody> {
        let comments = match &self.comments {
            None => return None,
            Some(c) => c,
        };
        match self.state.selected() {
            Some(i) => comments.comments.get(i),
            None => None,
        }
    }

    pub fn comment_contents_down(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_add(lines);
        if self.scroll >= 100 {
            self.scroll = 0
        }
    }

    pub fn comment_contents_up(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    #[allow(dead_code)]
    pub async fn update(&mut self, comments: Comments) -> anyhow::Result<()> {
        self.comments = Some(comments);
        Ok(())
    }
}

impl Component for CommentsList {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Some(action) = self.key_mappings.get(&key) {
            use Action::*;
            match *action {
                PageDown(line) => self.comment_contents_down(line),
                PageUp(line) => self.comment_contents_up(line),
                NextComment(line) => self.next(line),
                PreviousComment(line) => self.previous(line),
                LastComment => self.go_to_bottom(),
                FirstComment => self.go_to_top(),
            }
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}
