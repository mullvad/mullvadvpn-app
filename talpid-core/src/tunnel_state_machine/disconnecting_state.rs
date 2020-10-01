use super::{
    connecting_state::TunnelCloseEvent, ConnectingState, DisconnectedState, ErrorState,
    EventConsequence, SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver, TunnelState,
    TunnelStateTransition, TunnelStateWrapper,
};
use crate::tunnel::CloseHandle;
use futures::{future::FusedFuture, StreamExt};
use std::thread;
use talpid_types::{
    tunnel::{ActionAfterDisconnect, ErrorStateCause},
    ErrorExt,
};

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
            },
            AfterDisconnect::Block(reason) => match command {
                Some(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Block(reason)
                }
                Some(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if !is_offline && reason == ErrorStateCause::IsOffline {
                        AfterDisconnect::Reconnect(0)
                    } else {
                        AfterDisconnect::Block(reason)
                    }
                }
                Some(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Some(TunnelCommand::Disconnect) => AfterDisconnect::Nothing,
                Some(TunnelCommand::Block(new_reason)) => AfterDisconnect::Block(new_reason),
                None => AfterDisconnect::Block(reason),
            },
            AfterDisconnect::Reconnect(retry_attempt) => match command {
                Some(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
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

#[async_trait::async_trait]
impl TunnelState for DisconnectingState {
    type Bootstrap = (Option<CloseHandle>, TunnelCloseEvent, AfterDisconnect);

    fn enter(
        _: &mut SharedTunnelStateValues,
        (close_handle, tunnel_close_event, after_disconnect): Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if let Some(close_handle) = close_handle {
            thread::spawn(move || {
                if let Err(error) = close_handle.close() {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to close the tunnel")
                    );
                }
            });
        }

        let action_after_disconnect = after_disconnect.action();

        (
            TunnelStateWrapper::from(DisconnectingState {
                tunnel_close_event,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    async fn handle_event(
        mut self,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        if self.tunnel_close_event.is_terminated() {
            return NewState(self.after_disconnect(None, shared_values));
        }

        return futures::select! {
            command = commands.next() => {
                self.handle_commands(command, shared_values)
            }
            block_reason = &mut self.tunnel_close_event => {
                let block_reason = block_reason.unwrap_or(None);
                NewState(self.after_disconnect(block_reason, shared_values))
            }
        };
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
