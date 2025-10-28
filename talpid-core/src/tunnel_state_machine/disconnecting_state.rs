use super::{
    ConnectingState, DisconnectedState, ErrorState, EventConsequence, EventResult,
    SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver, TunnelState,
    TunnelStateTransition, connecting_state::TunnelCloseEvent,
};
use futures::{StreamExt, channel::oneshot, future::FusedFuture};
use talpid_types::tunnel::{ActionAfterDisconnect, ErrorStateCause};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    tunnel_close_event: TunnelCloseEvent,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    pub(super) fn enter(
        tunnel_close_tx: oneshot::Sender<()>,
        tunnel_close_event: TunnelCloseEvent,
        after_disconnect: AfterDisconnect,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        let _ = tunnel_close_tx.send(());
        let action_after_disconnect = after_disconnect.action();

        (
            Box::new(DisconnectingState {
                tunnel_close_event,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    fn handle_commands(
        mut self: Box<Self>,
        command: Option<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        match command {
            Some(TunnelCommand::AllowLan(allow_lan, complete_tx)) => {
                let _ = shared_values.set_allow_lan(allow_lan);
                let _ = complete_tx.send(());
            }
            #[cfg(not(target_os = "android"))]
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                shared_values.allowed_endpoint = endpoint;
                let _ = tx.send(());
            }
            Some(TunnelCommand::Dns(servers, complete_tx)) => {
                let _ = shared_values.set_dns_config(servers);
                let _ = complete_tx.send(());
            }
            #[cfg(not(target_os = "android"))]
            Some(TunnelCommand::LockdownMode(lockdown_mode, complete_tx)) => {
                shared_values.lockdown_mode = lockdown_mode;
                let _ = complete_tx.send(());
            }
            Some(TunnelCommand::Connectivity(connectivity)) => {
                shared_values.connectivity = connectivity;

                match self.after_disconnect {
                    AfterDisconnect::Reconnect(_) if connectivity.is_offline() => {
                        self.after_disconnect = AfterDisconnect::Block(ErrorStateCause::IsOffline)
                    }
                    AfterDisconnect::Block(ErrorStateCause::IsOffline)
                        if !connectivity.is_offline() =>
                    {
                        self.after_disconnect = AfterDisconnect::Reconnect(0);
                    }
                    _ => {}
                }
            }
            Some(TunnelCommand::Connect) => {
                self.after_disconnect = match self.after_disconnect {
                    AfterDisconnect::Reconnect(retry_attempt) => {
                        AfterDisconnect::Reconnect(retry_attempt)
                    }
                    _ => AfterDisconnect::Reconnect(0),
                };
            }
            Some(TunnelCommand::Disconnect) => {
                self.after_disconnect = AfterDisconnect::Nothing;
            }
            Some(TunnelCommand::Block(reason)) => {
                self.after_disconnect = match self.after_disconnect {
                    AfterDisconnect::Nothing => AfterDisconnect::Nothing,
                    _ => AfterDisconnect::Block(reason),
                }
            }
            None => {
                if let AfterDisconnect::Reconnect(_) = self.after_disconnect {
                    self.after_disconnect = AfterDisconnect::Nothing;
                }
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.exclude_paths(paths, result_tx);
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.set_excluded_paths(paths);
                let _ = result_tx.send(Ok(()));
            }
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                let _ = result_tx.send(shared_values.set_exclude_paths(paths).map(|_| ()));
            }
        };

        EventConsequence::SameState(self)
    }

    fn after_disconnect(
        self,
        block_reason: Option<ErrorStateCause>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        // Stop the WireGuard go runtime.
        // TODO: Assert that there are no other live handles.
        if let Some(wg_runtime) = shared_values.wg_runtime.take() {
            drop(wg_runtime)
        }

        if let Some(reason) = block_reason {
            return ErrorState::enter(shared_values, reason);
        }

        match self.after_disconnect {
            AfterDisconnect::Nothing => DisconnectedState::enter(shared_values, true),
            AfterDisconnect::Block(cause) => ErrorState::enter(shared_values, cause),
            AfterDisconnect::Reconnect(retry_attempt) => {
                ConnectingState::enter(shared_values, retry_attempt)
            }
        }
    }
}

impl TunnelState for DisconnectingState {
    fn handle_event(
        mut self: Box<Self>,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        let result = if self.tunnel_close_event.is_terminated() {
            if commands.is_done() {
                EventResult::Close(Ok(None))
            } else if let Ok(command) = commands.get_mut().try_next() {
                EventResult::Command(command)
            } else {
                EventResult::Close(Ok(None))
            }
        } else {
            runtime.block_on(async {
                futures::select! {
                    command = commands.next() => EventResult::Command(command),
                    result = &mut self.tunnel_close_event => EventResult::Close(result),
                }
            })
        };

        match result {
            EventResult::Command(command) => self.handle_commands(command, shared_values),
            EventResult::Close(result) => {
                let block_reason = result.unwrap_or(None);
                NewState(self.after_disconnect(block_reason, shared_values))
            }
            _ => unreachable!("unexpected event result"),
        }
    }
}

/// Which state should be transitioned to after disconnection is complete.
pub enum AfterDisconnect {
    Nothing,
    Block(ErrorStateCause),
    Reconnect(u32),
}

impl AfterDisconnect {
    /// Build event representation of the action that will be taken after the disconnection.
    pub fn action(&self) -> ActionAfterDisconnect {
        match self {
            AfterDisconnect::Nothing => ActionAfterDisconnect::Nothing,
            AfterDisconnect::Block(..) => ActionAfterDisconnect::Block,
            AfterDisconnect::Reconnect(..) => ActionAfterDisconnect::Reconnect,
        }
    }
}
