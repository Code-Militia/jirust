use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::{config::KeyConfig, events::key::Key};
use std::{char, collections::HashMap};

use crate::jira::tickets::CreateTicket;

use super::{draw_block_style, draw_edit_block_style, EventState, InputMode};

#[derive(Debug)]
pub enum FocusCreateTicket {
    Description,
    Summary,
    TicketType,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Edit,
    PushCreateTicketContent,
    Next,
    Previous,
}

#[derive(Debug)]
pub struct CreateTicketWidget {
    focus: FocusCreateTicket,
    input_mode: InputMode,
    ticket_type_state: TableState,
    pub contents: CreateTicket,
    pub push_content: bool,
    pub key_mappings: HashMap<Key, Action>,
}

impl CreateTicketWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let key_mappings = {
            let mut map = HashMap::new();
            map.insert(key_config.push, Action::PushCreateTicketContent);
            map.insert(key_config.next, Action::Next);
            map.insert(key_config.previous, Action::Previous);
            map.insert(key_config.edit, Action::Edit);
            map
        };

        let mut ticket_type_state = TableState::default();
        ticket_type_state.select(Some(0));
        Self {
            contents: CreateTicket::new(),
            focus: FocusCreateTicket::TicketType,
            input_mode: InputMode::Normal,
            push_content: false,
            key_mappings,
            ticket_type_state,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> anyhow::Result<()> {
        let constraints = [
            Constraint::Percentage(5),  // Helper
            Constraint::Percentage(20), // Ticket type and Summary
            Constraint::Percentage(75), // Description
        ];

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(f.size());

        let helper_constraint = [Constraint::Percentage(100)];
        let helper_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(helper_constraint)
            .split(main_chunks[0]);

        let type_and_summary_constraint = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let type_and_summary_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(type_and_summary_constraint)
            .split(main_chunks[1]);

        let description_constraint = [Constraint::Percentage(100)];
        let description_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(description_constraint)
            .split(main_chunks[2]);

        let ticket_type_chunk = type_and_summary_layout[0];
        let ticket_type_title = "Ticket Type";

        let summary_chunk = type_and_summary_layout[1];
        let summary_title = "Summary";

        let description_chunk = description_layout[0];
        let description_title = "Description";

        let normal_mode_style = (
            vec![
                Span::raw("Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
                Span::styled(" P", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to create ticket in JIRA."),
            ],
            Style::default().add_modifier(Modifier::UNDERLINED),
        );
        let edit_mode_style = (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
            ],
            Style::default(),
        );
        let (msg, style) = match self.input_mode {
            InputMode::Normal => normal_mode_style,
            InputMode::Editing => edit_mode_style,
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, helper_layout[0]);

        let ticket_type_headers_cells = ["Id", "Name", "Description"];
        let ticket_type_headers = Row::new(ticket_type_headers_cells);
        let ticket_type_row = self.contents.ticket_types.iter().map(|ticket_type| {
            let item = [ticket_type.id.as_str(), ticket_type.name.as_str()];
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().map(|c| Cell::from(*c));
            Row::new(cells).height(height as u16)
        });

        let ticket_type_table = Table::new(ticket_type_row)
            .header(ticket_type_headers)
            .style(draw_edit_block_style(
                matches!(self.focus, FocusCreateTicket::TicketType),
                &self.input_mode,
            ))
            .block(draw_block_style(
                matches!(self.focus, FocusCreateTicket::TicketType),
                ticket_type_title,
            ))
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(60),
            ]);
        f.render_stateful_widget(
            ticket_type_table,
            ticket_type_chunk,
            &mut self.ticket_type_state,
        );

        let summary_input = Paragraph::new(self.contents.summary.as_ref())
            .wrap(Wrap { trim: true })
            .style(draw_edit_block_style(
                matches!(self.focus, FocusCreateTicket::Summary),
                &self.input_mode,
            ))
            .block(draw_block_style(
                matches!(self.focus, FocusCreateTicket::Summary),
                summary_title,
            ));
        f.render_widget(summary_input, summary_chunk);

        let description_input = Paragraph::new(self.contents.description.as_ref())
            .wrap(Wrap { trim: true })
            .style(draw_edit_block_style(
                matches!(self.focus, FocusCreateTicket::Description),
                &self.input_mode,
            ))
            .block(draw_block_style(
                matches!(self.focus, FocusCreateTicket::Description),
                description_title,
            ));
        f.render_widget(description_input, description_chunk);

        Ok(())
    }

    pub fn input_pop(&mut self) {
        match self.focus {
            FocusCreateTicket::Description => {
                self.contents.description.pop();
            }
            FocusCreateTicket::Summary => {
                self.contents.summary.pop();
            }
            _ => {}
        };
    }
    pub fn input(&mut self, c: char) {
        match self.focus {
            FocusCreateTicket::Description => self.contents.description.push(c),
            FocusCreateTicket::Summary => self.contents.summary.push(c),
            _ => (),
        };
    }

    pub fn next_focus(&mut self) {
        match self.focus {
            FocusCreateTicket::TicketType => self.focus = FocusCreateTicket::Summary,
            FocusCreateTicket::Summary => self.focus = FocusCreateTicket::Description,
            FocusCreateTicket::Description => self.focus = FocusCreateTicket::TicketType,
        };
    }
    pub fn previous_focus(&mut self) {
        match self.focus {
            FocusCreateTicket::TicketType => self.focus = FocusCreateTicket::Description,
            FocusCreateTicket::Summary => self.focus = FocusCreateTicket::TicketType,
            FocusCreateTicket::Description => self.focus = FocusCreateTicket::Summary,
        };
    }
}

impl CreateTicketWidget {
    fn normal_mode_key_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Some(action) = self.key_mappings.get(&key) {
            use Action::*;
            match *action {
                Edit => {
                    self.input_mode = InputMode::Editing;
                }
                PushCreateTicketContent => {
                    self.push_content = true;
                }
                Next => self.next_focus(),
                Previous => self.previous_focus(),
            }
            Ok(EventState::Consumed)
        } else {
            Ok(EventState::NotConsumed)
        }
    }

    pub fn edit_mode_key_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match key {
            Key::Tab => {
                self.next_focus();
                Ok(EventState::Consumed)
            }
            Key::BackTab => {
                self.previous_focus();
                Ok(EventState::Consumed)
            }
            Key::Char(c) => {
                self.input(c);
                Ok(EventState::Consumed)
            }
            Key::Backspace => {
                self.input_pop();
                Ok(EventState::Consumed)
            }
            Key::Esc => {
                self.input_mode = InputMode::Normal;
                Ok(EventState::Consumed)
            }
            _ => Ok(EventState::NotConsumed),
        }
    }
    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.input_mode {
            InputMode::Normal => self.normal_mode_key_event(key),
            InputMode::Editing => self.edit_mode_key_event(key),
        }
    }
}
