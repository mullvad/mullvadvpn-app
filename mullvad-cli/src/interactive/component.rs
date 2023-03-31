use crossterm::event::Event;
use tui::{backend::CrosstermBackend, layout::Rect, Frame as TuiFrame};

pub type Backend = CrosstermBackend<std::io::Stdout>;
pub type Frame<'a> = TuiFrame<'a, Backend>;

pub trait Component {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect);
    fn handle_event(&mut self, _event: Event) {}
}
