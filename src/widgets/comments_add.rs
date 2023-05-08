use crate::event::key::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::{EventState, InputMode};

// CommentPopup holds the state of the application
#[derive(Debug)]
pub struct CommentAdd {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<String>,
    pub push_comment: bool,
}

impl CommentAdd {
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> anyhow::Result<()> {
        let chunk_constraints = [
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(1),
        ]
        .as_ref();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(chunk_constraints)
            .split(f.size());
        let input_title = "Add comments";

        let normal_mode_style = (
            vec![
                Span::raw("Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
                Span::styled("P", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to push comments to jira."),
            ],
            Style::default().add_modifier(Modifier::UNDERLINED),
        );
        let edit_mode_style = (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
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
        f.render_widget(help_message, chunks[0]);

        let input = Paragraph::new(self.input.as_ref())
            .wrap(Wrap { trim: true })
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title(input_title));
        f.render_widget(input, chunks[1]);

        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        let cursor_end_of_text = (
            // Put cursor past the end of the input text
            chunks[1].x + self.input.len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        );
        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => f.set_cursor(cursor_end_of_text.0, cursor_end_of_text.1),
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
            .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, chunks[2]);
        Ok(())
    }
}

impl CommentAdd {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            push_comment: false,
        }
    }

    pub fn edit_mode(&mut self) {
        self.input_mode = InputMode::Editing
    }

    pub fn normal_mode(&mut self) {
        self.input_mode = InputMode::Normal
    }

    fn normal_mode_key_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match key {
            Key::Char('e') => {
                self.edit_mode();
                Ok(EventState::Consumed)
            }
            Key::Char('P') => {
                self.push_comment = true;
                Ok(EventState::Consumed)
            }
            _ => Ok(EventState::NotConsumed),
        }
    }

    fn edit_mode_key_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match key {
            Key::Char(c) => {
                self.input.push(c);
                Ok(EventState::Consumed)
            }
            Key::Backspace => {
                self.input.pop();
                Ok(EventState::Consumed)
            }
            Key::Esc => {
                self.normal_mode();
                Ok(EventState::Consumed)
            }
            Key::Enter => {
                self.messages.push(self.input.clone());
                self.input.clear();
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
