use super::app::AppActions;
use crate::interactive::component::{Component, Frame};

use crossterm::event::Event;
use tui::{layout::Rect, widgets::Paragraph};

pub struct SelectLocation {
    actions: AppActions,
}

impl SelectLocation {
    pub fn new(actions: AppActions) -> Self {
        Self {
            actions: actions.clone(),
        }
    }
}

impl Component for SelectLocation {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let close_area = Rect::new(area.x + 1, area.y, 7, 1);
        let title_area = Rect::new(area.x + area.width / 2 - 7, area.y, area.width, 1);

        f.render_widget(Paragraph::new("X (esc)"), close_area);
        f.render_widget(Paragraph::new("Select location"), title_area);
    }

    fn handle_event(&mut self, event: Event) {}
}
