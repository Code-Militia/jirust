use tui::{backend::Backend, Frame, layout::Rect};
use std::io::Error;
pub mod projects;

pub trait StatefulDrawableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> Result<(), Error>;
}
