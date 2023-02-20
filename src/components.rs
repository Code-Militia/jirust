pub mod commands;
pub mod error;
pub mod tickets;
pub mod projects;

use commands::CommandInfo;

use async_trait::async_trait;
use tui::{backend::Backend, layout::{Rect, Layout}, Frame};

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

// pub trait TicketDrawableComponent {
//     fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> anyhow::Result<()>;
//     fn draw_metadata<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()>;
//     fn draw_description<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()>;
//     fn draw_work_log<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()>;
// }

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

    // async fn async_event(
    //     &mut self,
    //     _key: crate::event::key::Key,
    //     _pool: &Box<dyn Pool>, // TODO: change this to issues pool instead of databases
    // ) -> Result<EventState> {
    //     Ok(EventState::NotConsumed)
    // }

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
