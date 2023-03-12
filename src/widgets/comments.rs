use crate::{
    event::key::Key,
    jira::tickets::Comments,
};
use html2md::parse_html;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Clear, Cell, Table, TableState, Row},
    Frame,
};

use crate::config::KeyConfig;

use super::{draw_block_style, draw_highlight_style, EventState};

#[derive(Debug)]
pub struct CommentsWidget {
    pub comments: Option<Comments>,
    state: TableState,
    key_config: KeyConfig,
}

impl CommentsWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let title = "Comments";

        let header_cells = ["Author", "Updated Author", "Comment"];
        let headers = Row::new(header_cells);
        let rows = match &self.comments {
            None => return Ok(()),
            Some(c) => {
                c.comments.iter().map(|f| {
                    let item = [
                        f.author.display_name.as_str(),
                        f.update_author.display_name.as_str(),
                        f.rendered_body.as_str()
                    ];
                    let height = item
                        .iter()
                        .map(|content| content.chars().filter(|c| *c == '\n').count())
                        .max()
                        .unwrap_or(0)
                        + 1;
                    let cells = item.iter().map(|c| Cell::from(*c));
                    Row::new(cells).height(height as u16)
                })
            }
        };
        let table = Table::new(rows)
            .header(headers)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style())
            .widths(&[
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]);

        f.render_widget(Clear, rect);
        f.render_stateful_widget(table, rect, &mut self.state);

        Ok(())
    }
}

impl CommentsWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        return Self {
            comments: None,
            key_config,
            state
        };
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
        return Ok(EventState::NotConsumed);
    }
}
