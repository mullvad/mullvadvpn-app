use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use super::super::{oneshot_send, CommandResult};
use crate::{
    Daemon, DaemonCommand, Error, EventListener, ExcludedPathsUpdate, InternalDaemonEvent,
    ResponseTx,
};
use futures::channel::oneshot;
use mullvad_types::settings::Settings;
use talpid_core::{mpsc::Sender, tunnel_state_machine::TunnelCommand};
use talpid_types::ErrorExt;

impl<L: EventListener + Send + Clone + 'static> Daemon<L> {
    pub(in super::super) async fn handle_split_tunnel_command(
        &mut self,
        command: DaemonCommand,
    ) -> CommandResult {
        use DaemonCommand::*;
        match command {
            AddSplitTunnelApp(tx, path) => self.on_add_split_tunnel_app(tx, path).await,
            RemoveSplitTunnelApp(tx, path) => self.on_remove_split_tunnel_app(tx, path).await,
            ClearSplitTunnelApps(tx) => self.on_clear_split_tunnel_apps(tx).await,
            SetSplitTunnelState(tx, enabled) => self.on_set_split_tunnel_state(tx, enabled).await,
            cmd => return CommandResult::NotHandled(cmd),
        }
        CommandResult::Handled
    }

    /// Update the split app paths in both the settings and tunnel
    async fn set_split_tunnel_paths(
        &mut self,
        tx: ResponseTx<(), Error>,
        response_msg: &'static str,
        settings: Settings,
        update: ExcludedPathsUpdate,
    ) {
        let new_list = match update {
            ExcludedPathsUpdate::SetPaths(ref paths) => {
                if *paths == settings.split_tunnel.apps {
                    oneshot_send(tx, Ok(()), response_msg);
                    return;
                }
                paths.iter()
            }
            ExcludedPathsUpdate::SetState(_) => settings.split_tunnel.apps.iter(),
        };
        let new_state = match update {
            ExcludedPathsUpdate::SetPaths(_) => settings.split_tunnel.enable_exclusions,
            ExcludedPathsUpdate::SetState(state) => {
                if state == settings.split_tunnel.enable_exclusions {
                    oneshot_send(tx, Ok(()), response_msg);
                    return;
                }
                state
            }
        };

        if new_state || new_state != settings.split_tunnel.enable_exclusions {
            let tunnel_list = if new_state {
                new_list.map(|s| OsString::from(s)).collect()
            } else {
                vec![]
            };

            let (result_tx, result_rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::SetExcludedApps(result_tx, tunnel_list));
            let daemon_tx = self.tx.clone();

            tokio::spawn(async move {
                match result_rx.await {
                    Ok(Ok(_)) => (),
                    Ok(Err(error)) => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to set excluded apps list")
                        );
                        oneshot_send(tx, Err(Error::SplitTunnelError(error)), response_msg);
                        return;
                    }
                    Err(_) => {
                        log::error!("The tunnel failed to return a result");
                        return;
                    }
                }

                let _ = daemon_tx.send(InternalDaemonEvent::ExcludedPathsEvent(update, tx));
            });
        } else {
            let _ = self
                .tx
                .send(InternalDaemonEvent::ExcludedPathsEvent(update, tx));
        }
    }

    async fn on_add_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, path: PathBuf) {
        let settings = self.settings.to_settings();

        let mut new_list = settings.split_tunnel.apps.clone();
        new_list.insert(path);

        self.set_split_tunnel_paths(
            tx,
            "add_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        )
        .await;
    }

    async fn on_remove_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, path: PathBuf) {
        let settings = self.settings.to_settings();

        let mut new_list = settings.split_tunnel.apps.clone();
        new_list.remove(&path);

        self.set_split_tunnel_paths(
            tx,
            "remove_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        )
        .await;
    }

    async fn on_clear_split_tunnel_apps(&mut self, tx: ResponseTx<(), Error>) {
        let settings = self.settings.to_settings();
        let new_list = HashSet::new();
        self.set_split_tunnel_paths(
            tx,
            "clear_split_tunnel_apps response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        )
        .await;
    }

    async fn on_set_split_tunnel_state(&mut self, tx: ResponseTx<(), Error>, state: bool) {
        let settings = self.settings.to_settings();
        self.set_split_tunnel_paths(
            tx,
            "set_split_tunnel_state response",
            settings,
            ExcludedPathsUpdate::SetState(state),
        )
        .await;
    }
}
