use tui::{backend::Backend, Frame, layout::Rect};
pub mod projects;

pub trait DrawableComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect, focused: bool) -> anyhow::Result<()>;
}

pub trait StatefulDrawableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> anyhow::Result<()>;
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

