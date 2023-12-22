use tui::{Frame, backend::Backend, layout::{Constraint, Layout, Direction}, style::{Style, Modifier}, text::{Span, Text, Spans}, widgets::{Paragraph, Wrap}};

use crate::{config::KeyConfig, events::key::Key};
use std::{collections::HashMap, char};

use crate::jira::tickets::CreateTicket;

use super::{EventState, InputMode, draw_edit_block_style, draw_block_style};

#[derive(Debug)]
pub enum FocusCreateTicket {
    Description,
    ProjectID,
    Summary,
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

        Self {
            contents: CreateTicket::new(),
            focus: FocusCreateTicket::ProjectID,
            input_mode: InputMode::Normal,
            push_content: false,
            key_mappings,
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> anyhow::Result<()> {
        let chunk_constraints = [
            Constraint::Length(1), // Helper message
            Constraint::Length(4), // ProjectID
            Constraint::Length(7), // Summary
            Constraint::Min(1), // Description
        ]
        .as_ref();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(chunk_constraints)
            .split(f.size());
        let helper_chunk = chunks[0];

        let project_id_chunk = chunks[1];
        let project_id_title = "Project ID";

        let summary_chunk = chunks[2];
        let summary_title = "Summary";

        let description_chunk = chunks[3];
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
                Span::raw(" to stop editing, ")
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
        f.render_widget(help_message, helper_chunk);

        let project_id_input = Paragraph::new(self.contents.project_id.as_ref())
            .wrap(Wrap { trim: true })
            .style(draw_edit_block_style(matches!(self.focus, FocusCreateTicket::ProjectID), &self.input_mode))
            .block(draw_block_style(matches!(self.focus, FocusCreateTicket::ProjectID), project_id_title));
        f.render_widget(project_id_input, project_id_chunk);

        let summary_input = Paragraph::new(self.contents.summary.as_ref())
            .wrap(Wrap { trim: true })
            .style(draw_edit_block_style(matches!(self.focus, FocusCreateTicket::Summary), &self.input_mode))
            .block(draw_block_style(matches!(self.focus, FocusCreateTicket::Summary), summary_title));
        f.render_widget(summary_input, summary_chunk);

        let description_input = Paragraph::new(self.contents.description.as_ref())
            .wrap(Wrap { trim: true })
            .style(draw_edit_block_style(matches!(self.focus, FocusCreateTicket::Description), &self.input_mode))
            .block(draw_block_style(matches!(self.focus, FocusCreateTicket::Description), description_title));
        f.render_widget(description_input, description_chunk);


        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        let cursor_end_of_text = match self.focus {
                FocusCreateTicket::ProjectID => (
                    project_id_chunk.x + self.contents.project_id.len() as u16 + 1, project_id_chunk.y + 1
                ),
                FocusCreateTicket::Summary => (
                    summary_chunk.x + self.contents.summary.len() as u16 + 1, summary_chunk.y + 1
                ),
                FocusCreateTicket::Description => (
                    description_chunk.x + self.contents.description.len() as u16 + 1, description_chunk.y + 1
                )
            };
        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => f.set_cursor(cursor_end_of_text.0, cursor_end_of_text.1),
        }
        Ok(())
    }

    pub fn input_pop(&mut self) {
        match self.focus {
            FocusCreateTicket::ProjectID => self.contents.project_id.pop(),
            FocusCreateTicket::Description => self.contents.description.pop(),
            FocusCreateTicket::Summary => self.contents.summary.pop(),
        };
    }
    pub fn input(&mut self, c: char) {
        match self.focus {
            FocusCreateTicket::ProjectID => self.contents.project_id.push(c),
            FocusCreateTicket::Description => self.contents.description.push(c),
            FocusCreateTicket::Summary => self.contents.summary.push(c),
        }; 
    }

    pub fn next_focus(&mut self) {
        match self.focus {
            FocusCreateTicket::ProjectID => self.focus = FocusCreateTicket::Summary,
            FocusCreateTicket::Summary => self.focus = FocusCreateTicket::Description,
            FocusCreateTicket::Description=> self.focus = FocusCreateTicket::ProjectID,
        };
    }
    pub fn previous_focus(&mut self) {
        match self.focus {
            FocusCreateTicket::ProjectID => self.focus = FocusCreateTicket::Description,
            FocusCreateTicket::Summary => self.focus = FocusCreateTicket::ProjectID,
            FocusCreateTicket::Description=> self.focus = FocusCreateTicket::Summary,
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
            _ => Ok(EventState::NotConsumed)
        }
    }
    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.input_mode {
            InputMode::Normal => self.normal_mode_key_event(key),
            InputMode::Editing => self.edit_mode_key_event(key),
        }
    }
}
