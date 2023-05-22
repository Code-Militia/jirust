use crate::event::key::Key;
use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, EventState};

#[derive(Debug)]
pub struct DescriptionWidget {
    key_config: KeyConfig,
    scroll: u16,
}

impl DescriptionWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);
        let summary_title = "Summary";
        let title = "Description";

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let summary_text = ticket.fields.summary.clone();
        let mut text = ticket.rendered_fields.description.clone();
        text = parse_html(&text);

        let summary_paragraph = Paragraph::new(summary_text)
            .block(draw_block_style(focused, summary_title))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        let paragraph = Paragraph::new(text)
            .block(draw_block_style(focused, title))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Percentage(85)])
            .split(rect);

        f.render_widget(summary_paragraph, main_chunks[0]);
        f.render_widget(paragraph, main_chunks[1]);

        Ok(())
    }
}

impl DescriptionWidget {
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
}

impl DescriptionWidget {
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
