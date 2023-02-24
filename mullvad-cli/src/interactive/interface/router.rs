use super::{
    app::AppActions, main_view::MainView, select_location::SelectLocationContainer,
    tunnel_state_provider::TunnelStateBroadcast,
};
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode};
use mullvad_management_interface::ManagementServiceClient;
use tui::layout::Rect;

enum Route {
    MainView,
    SelectLocation,
}

pub struct Router {
    main_view: MainView,
    select_location: SelectLocationContainer,
    route: Route,
}

impl Router {
    pub fn new(
        actions: AppActions,
        rpc: ManagementServiceClient,
        tunnel_state_broadcast: TunnelStateBroadcast,
    ) -> Self {
        let main_view = MainView::new(actions.clone(), rpc.clone(), tunnel_state_broadcast);
        let select_location = SelectLocationContainer::new(actions, rpc);
        Self {
            main_view,
            select_location,
            route: Route::MainView,
        }
    }

    fn handle_main_view_event(&mut self, event: Event) {
        if matches!(event, Event::Key(event) if event.code == KeyCode::Char('l')) {
            self.route = Route::SelectLocation;
        } else {
            self.main_view.handle_event(event);
        }
    }

    fn handle_select_location_event(&mut self, event: Event) {
        if matches!(event, Event::Key(event) if event.code == KeyCode::Esc) {
            self.route = Route::MainView;
        } else {
            self.select_location.handle_event(event);
        }
    }
}

impl Component for Router {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        match self.route {
            Route::MainView => self.main_view.draw(f, area),
            Route::SelectLocation => self.select_location.draw(f, area),
        }
    }

    fn handle_event(&mut self, event: Event) {
        match self.route {
            Route::MainView => self.handle_main_view_event(event),
            Route::SelectLocation => self.handle_select_location_event(event),
        }
    }
}
