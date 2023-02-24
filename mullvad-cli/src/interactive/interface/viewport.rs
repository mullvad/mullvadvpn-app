use super::{app::AppActions, router::Router, tunnel_state_provider::TunnelStateBroadcast};
use crate::interactive::component::{Component, Frame};

use crossterm::event::Event;
use mullvad_management_interface::ManagementServiceClient;
use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 25;

pub struct Viewport {
    router: Router,
}

impl Viewport {
    pub fn new(
        actions: AppActions,
        rpc: ManagementServiceClient,
        tunnel_state_broadcast: TunnelStateBroadcast,
    ) -> Self {
        let router = Router::new(actions, rpc, tunnel_state_broadcast);
        Self { router }
    }
}

impl Component for Viewport {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let x = area.width.saturating_sub(WIDTH) / 2;
        let y = area.height.saturating_sub(HEIGHT) / 2;
        let width = WIDTH.clamp(0, area.width);
        let height = HEIGHT.clamp(0, area.height);
        let viewport_area = Rect::new(x + 1, y + 1, width, height);

        let border = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded);
        let border_area = Rect::new(x, y, width + 2, height + 2);

        self.router.draw(f, viewport_area);
        f.render_widget(border, border_area);
    }

    fn handle_event(&mut self, event: Event) {
        self.router.handle_event(event);
    }
}
