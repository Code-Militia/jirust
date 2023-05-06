use crate::{
    event::key::Key,
    jira::tickets::{CommentBody, Comments},
};
use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::config::KeyConfig;

use super::{draw_block_style, draw_highlight_style, EventState, commands::CommandInfo, Component};

#[derive(Debug)]
pub struct CommentContents {
    key_config: KeyConfig,
    scroll: u16,
}
impl CommentContents {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        comment: Option<&CommentBody>,
        focused: bool,
    ) -> anyhow::Result<()> {
        let comment = match comment {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };
        let size = f.size();
        let title = &comment.author.display_name;
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(size);

        let text = parse_html(&comment.rendered_body.clone());
        let paragraph = Paragraph::new(Span::styled(text, Style::default()))
            .alignment(Alignment::Left)
            .block(draw_block_style(focused, title))
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, size);
        f.render_widget(paragraph, chunks[0]);
        Ok(())
    }
}

impl CommentContents {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            key_config,
            scroll: 0,
        }
    }

    pub fn down(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_add(lines);
        if self.scroll >= 100 {
            self.scroll = 0
        }
    }

    pub fn up(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_sub(lines);
    }
    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.down(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.up(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.down(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.up(10);
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

#[derive(Debug)]
pub struct CommentsList {
    pub comments: Option<Comments>,
    state: TableState,
    key_config: KeyConfig,
}

impl CommentsList {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
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

        f.render_widget(Clear, rect);
        f.render_stateful_widget(table, rect, &mut self.state);

        Ok(())
    }
}

impl CommentsList {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            comments: None,
            key_config,
            state,
        }
    }

    pub fn next(&mut self, line: usize) {
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
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
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

    pub async fn update(&mut self, comments: Comments) -> anyhow::Result<()> {
        self.comments = Some(comments);
        Ok(())
    }
}

impl Component for CommentsList {
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
        }
        Ok(EventState::NotConsumed)
    }
}
