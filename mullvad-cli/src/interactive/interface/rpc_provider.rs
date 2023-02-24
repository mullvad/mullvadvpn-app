use std::sync::Arc;

use crate::interactive::component::{Component, Frame};

use crossterm::event::Event;
use mullvad_management_interface::{new_rpc_client, ManagementServiceClient};
use parking_lot::Mutex;
use tui::layout::Rect;

#[derive(Clone)]
pub struct RpcProvider<T: Component + Send + 'static> {
    child: Arc<Mutex<Option<T>>>,
}

impl<T: Component + Send + 'static> RpcProvider<T> {
    pub fn new<F>(child_factory: F) -> Self
    where
        F: FnOnce(ManagementServiceClient) -> T + Send + 'static,
    {
        let child = Arc::new(Mutex::new(None));

        let async_child = child.clone();
        tokio::spawn(async move {
            let rpc = new_rpc_client().await.unwrap();
            let mut child = async_child.lock();
            *child = Some(child_factory(rpc));
        });

        Self { child }
    }
}

impl<T: Component + Send + 'static> Component for RpcProvider<T> {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        if let Some(ref mut child) = *self.child.lock() {
            child.draw(f, area);
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(ref mut child) = *self.child.lock() {
            child.handle_event(event);
        }
    }
}
