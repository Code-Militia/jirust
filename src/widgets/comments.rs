use crate::{event::key::Key, jira::tickets::Comments};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{config::KeyConfig, jira::tickets::TicketData};

use super::{draw_block_style, draw_highlight_style, EventState};

#[derive(Debug)]
pub struct CommentsWidget {
    key_config: KeyConfig,
    comments: Comments,
    state: ListState,
}

impl CommentsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
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
        match ticket.fields.comments {
            None => {
                let empty_paragraph = Paragraph::new(comment_as_string);
                f.render_widget(empty_paragraph, chunks[1]);
                return Ok(());
            }
            Some(c) => c.body.iter().map(|comment| {
                comment_as_string += format!(
                    "{}\t{}\n {}\n<hr>\n\n",
                    &c.author.display_name, &c.update_author.display_name, &comment.rendered_body
                )
                .as_str()
            }),
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

impl CommentsWidget {}

impl CommentsWidget {
    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        return Ok(EventState::NotConsumed);
    }
}
