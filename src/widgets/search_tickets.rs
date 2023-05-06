use simsearch::SimSearch;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{event::key::Key, jira::tickets::TicketData};

use super::{draw_highlight_style, EventState, InputMode};

pub struct SearchTicketsWidget {
    pub input: String,
    search_tickets: Vec<String>,
    state: ListState,
    tickets: Vec<String>,
    pub input_mode: InputMode,
}

impl SearchTicketsWidget {
    pub fn new() -> Self {
        let state = ListState::default();
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            search_tickets: Vec::new(),
            state,
            tickets: Vec::new(),
        }
    }

    pub fn normal_mode(&mut self) {
        self.input_mode = InputMode::Normal
    }
}

impl SearchTicketsWidget {
    fn draw_edit<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) -> anyhow::Result<()> {
        self.search_tickets.clear();
        let mut engine: SimSearch<usize> = SimSearch::new();

        for (index, project) in self.tickets.iter().enumerate() {
            engine.insert(index, project)
        }

        let results: Vec<_> = engine
            .search(&self.input)
            .into_iter()
            .map(|project_id| {
                let ticket = &self.tickets[project_id];
                self.search_tickets.push(ticket.to_string());
                ListItem::new(ticket.clone())
            })
            .collect();

        let tickets = List::new(results)
            .block(Block::default().borders(Borders::ALL).title("Tickets"))
            .highlight_style(draw_highlight_style());
        f.render_stateful_widget(tickets, r, &mut self.state);

        Ok(())
    }

    fn draw_normal<B: Backend>(&mut self, f: &mut Frame<B>, r: Rect) -> anyhow::Result<()> {
        let results: Vec<_> = self
            .tickets
            .iter()
            .map(|project_id| {
                // self.search_tickets.push(project_id.to_string());
                ListItem::new(project_id.clone())
            })
            .collect();
        let tickets = List::new(results)
            .block(Block::default().borders(Borders::ALL).title("Tickets"))
            .highlight_style(draw_highlight_style());

        f.render_widget(tickets, r);
        Ok(())
    }
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> anyhow::Result<()> {
        let chunk_constrains = [
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(1),
        ]
        .as_ref();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(chunk_constrains)
            .split(f.size());
        f.render_widget(Clear, chunks[2]);

        let input_title = "Search";

        let normal_mode_style = (
            vec![
                Span::raw("Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
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
            InputMode::Normal => {
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here

                self.draw_normal(f, chunks[2])?;
            }
            InputMode::Editing => {
                self.draw_edit(f, chunks[2])?;
                f.set_cursor(cursor_end_of_text.0, cursor_end_of_text.1)
            }
        }

        Ok(())
    }

    pub fn next(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i + line >= self.tickets.len() => Some(self.tickets.len() - 1),
            Some(i) => Some(i + line),
            None => Some(0),
        };

        self.state.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) if i <= line => Some(0),
            Some(i) => Some(i - line),
            None => None,
        };

        self.state.select(i);
    }

    pub fn update(&mut self, tickets: Vec<TicketData>) {
        self.tickets = tickets.into_iter().map(|ticket| ticket.key).collect();
    }

    pub fn selected(&self) -> Option<&String> {
        match self.state.selected() {
            Some(i) => self.search_tickets.get(i),
            None => None,
        }
    }
}

impl SearchTicketsWidget {
    // fn commands(&self, _out: &mut Vec<CommandInfo>) {}
    fn movement(&mut self, key: Key) -> anyhow::Result<EventState> {
        match key {
            Key::Down => {
                self.next(1);
                Ok(EventState::Consumed)
            }
            Key::Up => {
                self.previous(1);
                Ok(EventState::Consumed)
            }
            Key::Ctrl('d') => {
                self.next(10);
                Ok(EventState::Consumed)
            }
            Key::Ctrl('u') => {
                self.previous(10);
                Ok(EventState::Consumed)
            }
            _ => Ok(EventState::NotConsumed),
        }
    }

    fn normal_mode_key_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match key {
            Key::Char('e') => {
                self.input_mode = InputMode::Editing;
                Ok(EventState::Consumed)
            }
            // _ => return Ok(EventState::NotConsumed)
            _ => self.movement(key),
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
            _ => self.movement(key), // _ => return Ok(EventState::NotConsumed)
        }
    }

    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.input_mode {
            InputMode::Normal => self.normal_mode_key_event(key),
            InputMode::Editing => self.edit_mode_key_event(key),
        }
    }
}
