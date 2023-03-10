use std::fmt::Write;

use crate::{
    event::key::Key,
    jira::{auth::JiraClient, SurrealAny, tickets::{CommentBody, Comments}},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, draw_highlight_style, EventState};

#[derive(Debug)]
pub struct CommentsWidget {
    comments: Option<Comments>,
    key_config: KeyConfig,
    scroll: u16,
}

impl CommentsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        _focused: bool,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let size = f.size();

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(size);

        let title = "Comments";
        let title_paragraph = Paragraph::new(Span::styled(
            title,
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(title_paragraph, chunks[0]);

        let mut comment_as_string = String::new();
        match &ticket.fields.comments {
            None => {
                let empty_paragraph = Paragraph::new(comment_as_string);
                f.render_widget(empty_paragraph, chunks[1]);
                return Ok(());
            }
            Some(c) => for comment in &c.body {
                write!(
                    &mut comment_as_string,
                    "{}\t{}\n {}\n<hr>\n\n",
                    &c.author.display_name, &c.update_author.display_name, &comment.rendered_body
                )?;
            }
        };

        let comment_paragraph = Paragraph::new(Span::styled(
            comment_as_string,
            Style::default().add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(comment_paragraph, chunks[1]);

        Ok(())
    }
}

impl CommentsWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        return Self {
            comments: None,
            key_config,
            scroll: 0,
        };
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

    pub async fn update(
        &mut self,
        comments: Comments
    ) -> anyhow::Result<()> {
        self.comments = Some(comments);
        Ok(())
    }
}

impl CommentsWidget {
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
        return Ok(EventState::NotConsumed);
    }
}
