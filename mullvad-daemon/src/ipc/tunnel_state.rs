use super::oneshot_send;
use crate::{Daemon, EventListener};
use futures::channel::oneshot;
use mullvad_types::states::{TargetState, TunnelState};

impl<L: EventListener> Daemon<L> {
    pub(super) fn on_get_state(&mut self, tx: oneshot::Sender<TunnelState>) {
        oneshot_send(tx, self.tunnel_state.clone(), "current state");
    }
}

impl<L: EventListener + Send + Clone + 'static> Daemon<L> {
    pub(super) async fn on_set_target_state(
        &mut self,
        tx: oneshot::Sender<bool>,
        new_target_state: TargetState,
    ) {
        if self.state.is_running() {
            let state_change_initated = self.set_target_state(new_target_state).await;
            oneshot_send(tx, state_change_initated, "state change initiated");
        } else {
            log::warn!("Ignoring target state change request due to shutdown");
        }
    }

    pub(super) fn on_reconnect(&mut self, tx: oneshot::Sender<bool>) {
        if *self.target_state == TargetState::Secured || self.tunnel_state.is_in_error_state() {
            self.connect_tunnel();
            oneshot_send(tx, true, "reconnect issued");
        } else {
            log::debug!("Ignoring reconnect command. Currently not in secured state");
            oneshot_send(tx, false, "reconnect issued");
        }
    }
}
