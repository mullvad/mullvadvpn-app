use crate::interactive::component::{Component, Frame};

use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub struct Dialog {
    title: String,
    body: String,
}

impl Dialog {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_owned(),
            body: body.to_owned(),
        }
    }
}

impl Component for Dialog {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black));

        let dialog = Paragraph::new(self.body.as_str())
            .block(block)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        let area = Rect::new((area.width / 2) - 20, (area.height / 2) - 3, 40, 6);

        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }
}
