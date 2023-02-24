use std::sync::Arc;

use super::{app::AppActions, tunnel_state_provider::TunnelStateBroadcast};
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode, KeyEvent};
use mullvad_management_interface::ManagementServiceClient;
use mullvad_types::states::TunnelState;
use parking_lot::Mutex;
use tui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Clone)]
pub struct TunnelControl {
    state: Arc<Mutex<TunnelState>>,
    rpc: ManagementServiceClient,
}

impl TunnelControl {
    pub fn new(
        actions: AppActions,
        rpc: ManagementServiceClient,
        tunnel_state_broadcast: TunnelStateBroadcast,
    ) -> Self {
        let (tunnel_state, mut receiver) = tunnel_state_broadcast.receiver();
        let state = Arc::new(Mutex::new(
            tunnel_state.unwrap_or(TunnelState::Disconnected),
        ));

        let state2 = state.clone();
        tokio::spawn(async move {
            while let Some(new_state) = receiver.recv().await {
                {
                    let mut state = state2.lock();
                    *state = new_state;
                }

                actions.redraw_async().await;
            }
        });

        Self { state, rpc }
    }

    fn button_color(state: &TunnelState) -> Color {
        match state {
            TunnelState::Disconnected => Color::Green,
            _ => Color::Red,
        }
    }

    fn button_label(state: &TunnelState) -> String {
        match state {
            TunnelState::Disconnected => String::from("Secure my connection (enter)"),
            TunnelState::Connecting { .. } => String::from("Cancel (c)"),
            TunnelState::Connected { .. } => String::from("      Disconnect (d)"),
            TunnelState::Disconnecting(_) => String::from("Cancel (c)"),
            TunnelState::Error(_) => String::from("Dismiss (d)"),
        }
    }
}

impl Component for TunnelControl {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let state = self.state.lock();

        let mut button_area = Rect::new(area.x, area.y + 3, area.width, area.height - 3);
        if let TunnelState::Connected { .. } = *state {
            button_area = Rect::new(
                button_area.x,
                button_area.y,
                button_area.width - 7,
                button_area.height,
            );

            let reconnect_area = Rect::new(
                button_area.x + button_area.width,
                button_area.y,
                7,
                button_area.height,
            );
            let reconnect_button = Paragraph::new("(r)")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Self::button_color(&state))),
                )
                .alignment(Alignment::Center);

            f.render_widget(reconnect_button, reconnect_area);
        }

        let button = Paragraph::new(Self::button_label(&state))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Self::button_color(&state))),
            )
            .alignment(Alignment::Center);
        f.render_widget(button, button_area);

        let location_area = Rect::new(area.x, area.y, area.width, 3);
        let location_button = Paragraph::new("Select location (l)")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .alignment(Alignment::Center);
        f.render_widget(location_button, location_area);
    }

    fn handle_event(&mut self, event: Event) {
        let mut rpc = self.rpc.clone();
        tokio::spawn(async move {
            if let Event::Key(KeyEvent { code, .. }) = event {
                if code == KeyCode::Enter {
                    let _ = rpc.connect_tunnel(()).await;
                } else if let KeyCode::Char(character) = code {
                    match character {
                        'c' | 'd' => {
                            let _ = rpc.disconnect_tunnel(()).await;
                        }
                        'r' => {
                            let _ = rpc.reconnect_tunnel(()).await;
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}
