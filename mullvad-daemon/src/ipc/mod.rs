//! Handles commands exposed to front ends via [mullvad_management_interface].
use crate::{Daemon, DaemonCommand, EventListener};
use futures::channel::oneshot;

mod api;
mod geoip;
mod split_tunnel;
mod tunnel_state;

// TODO: Move all of `Daemon::handle_command` here.

impl<L: EventListener + Send + Clone + 'static> Daemon<L> {
    pub(super) async fn handle_ipc_command(&mut self, mut command: DaemonCommand) {
        use DaemonCommand::*;

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        match self.handle_split_tunnel_command(command).await {
            CommandResult::Handled => return,
            CommandResult::NotHandled(cmd) => command = cmd,
        }

        match command {
            GetCurrentLocation(tx) => self.on_get_current_location(tx),

            GetState(tx) => self.on_get_state(tx),
            SetTargetState(tx, state) => self.on_set_target_state(tx, state).await,
            Reconnect(tx) => self.on_reconnect(tx),

            GetWwwAuthToken(tx) => self.on_get_www_auth_token(tx),

            _ => unreachable!("handled by Daemon::handle_command"),
        }
    }
}

enum CommandResult {
    NotHandled(DaemonCommand),
    Handled,
}

fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
    if tx.send(t).is_err() {
        log::warn!("Unable to send {} to the daemon command sender", msg);
    }
}
