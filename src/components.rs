use tui::{backend::Backend, Frame, layout::Rect};
pub mod projects;

pub trait StatefulDrawableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> anyhow::Result<()>;
}
