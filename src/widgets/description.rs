use crate::event::key::Key;
use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, EventState};

#[derive(Debug)]
pub struct DescriptionWidget {
    description: String,
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
        let title = "Description";

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let mut text = ticket.rendered_fields.description.clone();
        text = parse_html(&text);

        let paragraph = Paragraph::new(text)
            .block(draw_block_style(focused, title))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        f.render_widget(paragraph, rect);

        Ok(())
    }
}

impl DescriptionWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        return Self {
            description: String::new(),
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
        // } else if key == self.key_config.scroll_to_bottom {
        //     self.go_to_bottom();
        //     return Ok(EventState::Consumed);
        // } else if key == self.key_config.scroll_to_top {
        //     self.go_to_top();
        //     return Ok(EventState::Consumed);
        // }
        return Ok(EventState::NotConsumed);
    }
}
