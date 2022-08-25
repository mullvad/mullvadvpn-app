use super::{
    connecting_state::TunnelCloseEvent, ConnectingState, DisconnectedState, ErrorState,
    EventConsequence, EventResult, SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use futures::{channel::oneshot, future::FusedFuture, StreamExt};
use talpid_types::tunnel::{ActionAfterDisconnect, ErrorStateCause};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    tunnel_close_event: TunnelCloseEvent,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    fn handle_commands(
        mut self,
        command: Option<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        let after_disconnect = self.after_disconnect;

        self.after_disconnect = match after_disconnect {
            AfterDisconnect::Nothing => match command {
                Some(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Nothing
                }
                Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                    shared_values.allowed_endpoint = endpoint;
                    let _ = tx.send(());
                    AfterDisconnect::Nothing
                }
                Some(TunnelCommand::Dns(servers)) => {
                    let _ = shared_values.set_dns_servers(servers);
                    AfterDisconnect::Nothing
                }
                Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Nothing
                }
                Some(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    AfterDisconnect::Nothing
                }
                Some(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Some(TunnelCommand::Disconnect) | None => AfterDisconnect::Nothing,
                Some(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
                #[cfg(target_os = "android")]
                Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                    shared_values.bypass_socket(fd, done_tx);
                    AfterDisconnect::Nothing
                }
                #[cfg(windows)]
                Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                    shared_values.split_tunnel.set_paths(&paths, result_tx);
                    AfterDisconnect::Nothing
                }
            },
            AfterDisconnect::Block(reason) => match command {
                Some(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                    shared_values.allowed_endpoint = endpoint;
                    let _ = tx.send(());
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::Dns(servers)) => {
                    let _ = shared_values.set_dns_servers(servers);
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if !is_offline && matches!(reason, ErrorStateCause::IsOffline) {
                        AfterDisconnect::Reconnect(0)
                    } else {
                        AfterDisconnect::Block(reason)
                    }
                }
                Some(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Some(TunnelCommand::Disconnect) => AfterDisconnect::Nothing,
                Some(TunnelCommand::Block(new_reason)) => AfterDisconnect::Block(new_reason),
                #[cfg(target_os = "android")]
                Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                    shared_values.bypass_socket(fd, done_tx);
                    AfterDisconnect::Block(reason)
                }
                #[cfg(windows)]
                Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                    shared_values.split_tunnel.set_paths(&paths, result_tx);
                    AfterDisconnect::Block(reason)
                }
                None => AfterDisconnect::Block(reason),
            },
            AfterDisconnect::Reconnect(retry_attempt) => match command {
                Some(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                    shared_values.allowed_endpoint = endpoint;
                    let _ = tx.send(());
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Some(TunnelCommand::Dns(servers)) => {
                    let _ = shared_values.set_dns_servers(servers);
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Some(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if is_offline {
                        AfterDisconnect::Block(ErrorStateCause::IsOffline)
                    } else {
                        AfterDisconnect::Reconnect(retry_attempt)
                    }
                }
                Some(TunnelCommand::Connect) => AfterDisconnect::Reconnect(retry_attempt),
                Some(TunnelCommand::Disconnect) | None => AfterDisconnect::Nothing,
                Some(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
                #[cfg(target_os = "android")]
                Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                    shared_values.bypass_socket(fd, done_tx);
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                #[cfg(windows)]
                Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                    shared_values.split_tunnel.set_paths(&paths, result_tx);
                    AfterDisconnect::Reconnect(retry_attempt)
                }
            },
        };

        EventConsequence::SameState(self.into())
    }

    fn after_disconnect(
        self,
        block_reason: Option<ErrorStateCause>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
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
    type Bootstrap = (oneshot::Sender<()>, TunnelCloseEvent, AfterDisconnect);

    fn enter(
        _: &mut SharedTunnelStateValues,
        (tunnel_close_tx, tunnel_close_event, after_disconnect): Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let _ = tunnel_close_tx.send(());
        let action_after_disconnect = after_disconnect.action();

        (
            TunnelStateWrapper::from(DisconnectingState {
                tunnel_close_event,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    fn handle_event(
        mut self,
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
