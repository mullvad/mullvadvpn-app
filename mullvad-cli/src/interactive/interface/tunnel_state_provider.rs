use std::sync::Arc;

use crate::{
    interactive::component::{Component, Frame},
    Result,
};

use crossterm::event::Event;
use mullvad_management_interface::{
    types::daemon_event::Event as EventType, ManagementServiceClient,
};
use mullvad_types::states::TunnelState;
use parking_lot::Mutex;
use tui::layout::Rect;

#[derive(Clone)]
pub struct TunnelStateProvider<T: Component> {
    child: T,
}

impl<T: Component> TunnelStateProvider<T> {
    pub fn new(
        child_factory: impl FnOnce(TunnelStateBroadcast) -> T,
        rpc: ManagementServiceClient,
    ) -> Self {
        let (state_sender, state_receiver) = tokio::sync::mpsc::unbounded_channel();
        let broadcast = TunnelStateBroadcast::new(state_receiver);
        let child = child_factory(broadcast);

        tokio::spawn(async move {
            let _ = Self::listen_tunnel_state(rpc, state_sender).await;
        });

        Self { child }
    }

    pub async fn listen_tunnel_state(
        mut rpc: ManagementServiceClient,
        sender: tokio::sync::mpsc::UnboundedSender<TunnelState>,
    ) -> Result<()> {
        let state = rpc.get_tunnel_state(()).await?.into_inner();
        let tunnel_state = TunnelState::try_from(state).unwrap();
        let _ = sender.send(tunnel_state);

        let mut events = rpc.events_listen(()).await?.into_inner();
        while let Some(event) = events.message().await? {
            if let Some(EventType::TunnelState(state)) = event.event {
                if let Ok(tunnel_state) = TunnelState::try_from(state) {
                    let _ = sender.send(tunnel_state);
                }
            }
        }

        Ok(())
    }
}

impl<T: Component> Component for TunnelStateProvider<T> {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.child.draw(f, area);
    }

    fn handle_event(&mut self, event: Event) {
        self.child.handle_event(event);
    }
}

#[derive(Clone)]
pub struct TunnelStateBroadcast {
    last_state: Arc<Mutex<Option<TunnelState>>>,
    senders: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<TunnelState>>>>,
}

impl TunnelStateBroadcast {
    pub fn new(mut receiver: tokio::sync::mpsc::UnboundedReceiver<TunnelState>) -> Self {
        let last_state = Arc::new(Mutex::new(None));
        let senders: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<TunnelState>>>> =
            Arc::new(Mutex::new(Vec::new()));

        let senders2 = senders.clone();
        let last_state2 = last_state.clone();
        tokio::spawn(async move {
            while let Some(state) = receiver.recv().await {
                {
                    let mut last_state = last_state2.lock();
                    *last_state = Some(state.clone());
                }

                let senders = senders2.lock();
                for sender in senders.iter() {
                    let _ = sender.send(state.clone());
                }
            }
        });

        Self {
            senders,
            last_state,
        }
    }

    pub fn receiver(
        &self,
    ) -> (
        Option<TunnelState>,
        tokio::sync::mpsc::UnboundedReceiver<TunnelState>,
    ) {
        let last_state = { self.last_state.lock().clone() };

        let (state_sender, state_receiver) = tokio::sync::mpsc::unbounded_channel();
        self.senders.lock().push(state_sender);

        (last_state, state_receiver)
    }
}
