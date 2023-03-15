use log::info;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Cell, Clear, ListState, Row, Table, TableState},
    Frame,
};

use crate::{
    config::KeyConfig,
    event::key::Key,
    jira::{
        auth::JiraClient,
        tickets::{JiraTickets, LinkInwardOutwardParent, Links, TicketData},
    },
};

use super::{commands::CommandInfo, draw_block_style, draw_highlight_style, Component, EventState};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct TicketParentWidget {}

impl TicketParentWidget {
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

        let mut rows = Vec::new();

        let title = "Parent";
        let header_cells = ["Key", "Summary", "Priority", "Type", "Status"];
        let headers = Row::new(header_cells);
        let ticket_parent = match &ticket.fields.parent {
            // None => return Ok(()),
            None => {
                let table = Table::new(rows)
                    .header(headers)
                    .block(draw_block_style(focused, &title))
                    .highlight_style(draw_highlight_style())
                    .widths(&[
                        Constraint::Percentage(15),
                        Constraint::Percentage(10),
                        Constraint::Percentage(15),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                    ]);

                f.render_widget(table, rect);
                return Ok(());
            }
            Some(i) => i,
            // _ => unreachable!("If there is a link it should be present")
        };
        let item = [
            ticket_parent.key.as_str(),
            ticket_parent.fields.summary.as_str(),
            ticket_parent.fields.priority.name.as_str(),
            ticket_parent.fields.issuetype.name.as_str(),
            ticket_parent.fields.status.name.as_str(),
        ];
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        // let rows = Row::new(cells).height(height as u16);
        rows.push(Row::new(cells).height(height as u16));
        let table = Table::new(rows)
            .header(headers)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style())
            .widths(&[
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]);

        f.render_widget(Clear, rect);
        f.render_widget(table, rect);

        Ok(())
    }
}

impl TicketParentWidget {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        return Self {};
    }
}
