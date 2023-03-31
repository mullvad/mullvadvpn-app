use std::sync::Arc;

use super::{
    app::AppActions, loading_indicator::LoadingIndicator, tunnel_control::TunnelControl,
    tunnel_state_provider::TunnelStateBroadcast,
};
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode};
use mullvad_types::{location::GeoIpLocation, states::TunnelState};
use parking_lot::Mutex;
use tui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

#[derive(Clone)]
pub struct MainView {
    actions: AppActions,
    state: Arc<Mutex<TunnelState>>,
    show_details: Arc<Mutex<bool>>,
    loading_indicator: LoadingIndicator,
    tunnel_control: TunnelControl,
}

impl MainView {
    pub fn new(actions: AppActions, tunnel_state_broadcast: TunnelStateBroadcast) -> Self {
        let (tunnel_state, mut receiver) = tunnel_state_broadcast.receiver();
        let state = Arc::new(Mutex::new(
            tunnel_state.unwrap_or(TunnelState::Disconnected),
        ));

        let actions2 = actions.clone();
        let state2 = state.clone();
        tokio::spawn(async move {
            while let Some(new_state) = receiver.recv().await {
                {
                    let mut state = state2.lock();
                    *state = new_state;
                }

                actions2.redraw_async().await;
            }
        });

        Self {
            actions: actions.clone(),
            state,
            show_details: Arc::new(Mutex::new(false)),
            loading_indicator: LoadingIndicator::new(actions.clone()),
            tunnel_control: TunnelControl::new(actions, tunnel_state_broadcast),
        }
    }

    fn header(state: &TunnelState) -> impl Widget {
        let color = Self::state_color(state);
        Paragraph::new("\n   👷  MULLVAD VPN")
            .block(Block::default().border_style(Style::default().bg(Color::White)))
            .style(Style::default().fg(Color::White).bg(color))
    }

    fn connection_info(state: &TunnelState) -> impl Widget {
        let status_label = Self::status_label(state);
        let status_label_color = Self::status_label_color(state);

        let mut list_items =
            vec![ListItem::new(status_label).style(Style::default().fg(status_label_color))];

        if let TunnelState::Connected {
            location: Some(location),
            ..
        } = state
        {
            Self::append_location_info(location, &mut list_items);
        }

        if let TunnelState::Connecting {
            location: Some(location),
            ..
        } = state
        {
            Self::append_location_info(location, &mut list_items);
        }

        List::new(list_items)
    }

    fn append_location_info(location: &GeoIpLocation, to: &mut Vec<ListItem<'_>>) {
        to.push(ListItem::new(location.country.clone()));
        to.push(ListItem::new(location.city.clone().unwrap_or_default()));
        to.push(
            ListItem::new(
                location
                    .hostname
                    .clone()
                    .map(|hostname| format!("{} (i)", hostname))
                    .unwrap_or_default(),
            )
            .style(Style::default().fg(Color::Gray)),
        );
    }

    fn connection_details(state: &TunnelState) -> impl Widget {
        let mut list_items = Vec::new();

        if let TunnelState::Connected { endpoint, location } = state {
            list_items.push(ListItem::new(format!("{:?}", endpoint.tunnel_type)));
            list_items.push(ListItem::new(format!("In: {}", endpoint.endpoint.address)));

            if let Some(GeoIpLocation {
                ipv4: Some(ipv4), ..
            }) = location
            {
                list_items.push(ListItem::new(format!("Out: {}", ipv4)));
            }
        }

        List::new(list_items).style(Style::default().fg(Color::Gray))
    }

    fn state_color(state: &TunnelState) -> Color {
        match state {
            TunnelState::Disconnected => Color::Red,
            TunnelState::Connecting { .. } => Color::Green,
            TunnelState::Connected { .. } => Color::Green,
            TunnelState::Disconnecting(_) => Color::Green,
            TunnelState::Error(details) => {
                if details.is_blocking() {
                    Color::Green
                } else {
                    Color::Red
                }
            }
        }
    }

    fn status_label(state: &TunnelState) -> String {
        match state {
            TunnelState::Disconnected => String::from("UNSECURE CONNECTION"),
            TunnelState::Connecting { .. } => String::from("CREATING SECURE CONNECTION"),
            TunnelState::Connected { .. } => String::from("SECURE CONNECTION"),
            TunnelState::Disconnecting(_) => String::from(""),
            TunnelState::Error(_) => String::from("FAILED TO SECURE CONNECTION"),
        }
    }

    fn status_label_color(state: &TunnelState) -> Color {
        match state {
            TunnelState::Disconnected => Color::Red,
            TunnelState::Connecting { .. } => Color::White,
            TunnelState::Connected { .. } => Color::Green,
            TunnelState::Disconnecting(_) => Color::White,
            TunnelState::Error(_) => Color::Red,
        }
    }
}

impl Component for MainView {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let state = self.state.lock();

        let header_area = Rect::new(area.x, area.y, area.width, 3);
        f.render_widget(Self::header(&state), header_area);

        if matches!(*state, TunnelState::Connecting { .. }) {
            let indicator_area = Rect::new(area.x, area.y + 5, area.width, 6);
            self.loading_indicator.draw(f, indicator_area);
        }

        let info_area = Rect::new(area.x + 4, area.y + area.height / 2 - 2, area.width - 6, 4);
        f.render_widget(Self::connection_info(&state), info_area);

        if *self.show_details.lock() {
            let details_area =
                Rect::new(area.x + 6, area.y + area.height / 2 + 2, area.width - 8, 3);
            f.render_widget(Self::connection_details(&state), details_area);
        }

        let control_area = Rect::new(area.x + 3, area.y + area.height - 7, area.width - 6, 6);
        self.tunnel_control.draw(f, control_area);
    }

    fn handle_event(&mut self, event: Event) {
        if matches!(event, Event::Key(event) if event.code == KeyCode::Char('i')) {
            let mut show_details = self.show_details.lock();
            *show_details = !*show_details;
            self.actions.redraw();
        } else {
            self.tunnel_control.handle_event(event);
        }
    }
}
