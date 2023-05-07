pub mod commands;
pub mod comments;
pub mod comments_add;
pub mod components;
pub mod description;
pub mod error;
pub mod help;
pub mod labels;
pub mod parent;
pub mod projects;
pub mod search_projects;
pub mod search_tickets;
pub mod ticket_relation;
pub mod ticket_transition;
pub mod tickets;

use commands::CommandInfo;

use async_trait::async_trait;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame,
};

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq, Debug)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

pub trait DrawableComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect, focused: bool) -> anyhow::Result<()>;
}

pub trait StatefulDrawableComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
    ) -> anyhow::Result<()>;
}

pub trait MovableComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
        x: u16,
        y: u16,
    ) -> anyhow::Result<()>;
}

/// base component trait
#[async_trait]
pub trait Component {
    fn commands(&self, out: &mut Vec<CommandInfo>);

    fn event(&mut self, key: crate::event::key::Key) -> anyhow::Result<EventState>;

    fn focused(&self) -> bool {
        false
    }

    fn focus(&mut self, _focus: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn hide(&mut self) {}

    fn show(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn toggle_visible(&mut self) -> anyhow::Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}

pub fn draw_block_style(focused: bool, title: &str) -> Block {
    if focused {
        Block::default()
            .border_type(BorderType::Double)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(title)
            .title_alignment(tui::layout::Alignment::Center)
    } else {
        Block::default()
            .border_type(BorderType::Plain)
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(tui::layout::Alignment::Center)
    }
}

pub fn draw_highlight_style() -> Style {
    Style::default().bg(Color::Blue)
}
