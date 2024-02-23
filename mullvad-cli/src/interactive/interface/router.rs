use std::sync::Arc;

use super::{
    app::AppActions, main_view::MainView, select_location::SelectLocationContainer,
    tunnel_state_provider::TunnelStateBroadcast,
};
use crate::interactive::component::{Component, Frame};

use crossterm::event::{Event, KeyCode};
use mullvad_management_interface::MullvadProxyClient;
use parking_lot::Mutex;
use tui::layout::Rect;

#[derive(Clone)]
pub enum Route {
    MainView,
    SelectLocation,
}

pub struct Router {
    actions: AppActions,
    main_view: MainView,
    select_location: SelectLocationContainer,
    route: Arc<Mutex<Route>>,
}

impl Router {
    pub fn new(
        actions: AppActions,
        rpc: MullvadProxyClient,
        tunnel_state_broadcast: TunnelStateBroadcast,
    ) -> Self {
        let main_view = MainView::new(actions.clone(), rpc.clone(), tunnel_state_broadcast);

        let (router_sender, router_receiver) = flume::unbounded::<Route>();
        let route = Arc::new(Mutex::new(Route::MainView));

        let actions2 = actions.clone();
        let route2 = route.clone();
        tokio::spawn(async move {
            while let Ok(new_route) = router_receiver.recv_async().await {
                {
                    let mut route = route2.lock();
                    *route = new_route;
                }
                actions2.redraw_async().await;
            }
        });

        let actions3 = actions.clone();
        let select_location = SelectLocationContainer::new(actions3, rpc, router_sender);
        Self {
            actions,
            main_view,
            select_location,
            route,
        }
    }

    fn handle_main_view_event(&mut self, event: Event) {
        if matches!(event, Event::Key(event) if event.code == KeyCode::Char('l')) {
            let actions = self.actions.clone();
            let route = self.route.clone();
            tokio::spawn(async move {
                {
                    let mut route = route.lock();
                    *route = Route::SelectLocation;
                }
                actions.redraw_async().await;
            });
        } else {
            self.main_view.handle_event(event);
        }
    }

    fn handle_select_location_event(&mut self, event: Event) {
        if matches!(event, Event::Key(event) if event.code == KeyCode::Esc) {
            let actions = self.actions.clone();
            let route = self.route.clone();
            tokio::spawn(async move {
                {
                    let mut route = route.lock();
                    *route = Route::MainView;
                }
                actions.redraw_async().await;
            });
        } else {
            self.select_location.handle_event(event);
        }
    }
}

impl Component for Router {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let route = self.route.lock();
        match *route {
            Route::MainView => self.main_view.draw(f, area),
            Route::SelectLocation => self.select_location.draw(f, area),
        }
    }

    fn handle_event(&mut self, event: Event) {
        let route = { self.route.lock().clone() };
        match route {
            Route::MainView => self.handle_main_view_event(event),
            Route::SelectLocation => self.handle_select_location_event(event),
        }
    }
}
