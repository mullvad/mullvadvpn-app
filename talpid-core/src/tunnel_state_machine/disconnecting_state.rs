use super::{
    ConnectingState, DisconnectedState, ErrorState, EventConsequence, SharedTunnelStateValues,
    TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::tunnel::CloseHandle;
use futures01::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use std::thread;
use talpid_types::{
    tunnel::{ActionAfterDisconnect, ErrorStateCause},
    ErrorExt,
};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: Option<oneshot::Receiver<Option<ErrorStateCause>>>,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        let event = try_handle_event!(self, commands.poll());
        let after_disconnect = self.after_disconnect;

        self.after_disconnect = match after_disconnect {
            AfterDisconnect::Nothing => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Ok(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
                _ => AfterDisconnect::Nothing,
            },
            AfterDisconnect::Block(reason) => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Block(reason)
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Block(reason)
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if !is_offline && reason == ErrorStateCause::IsOffline {
                        AfterDisconnect::Reconnect(0)
                    } else {
                        AfterDisconnect::Block(reason)
                    }
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Ok(TunnelCommand::Disconnect) => AfterDisconnect::Nothing,
                Ok(TunnelCommand::Block(new_reason)) => AfterDisconnect::Block(new_reason),
                Err(_) => AfterDisconnect::Block(reason),
            },
            AfterDisconnect::Reconnect(retry_attempt) => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    let _ = shared_values.set_allow_lan(allow_lan);
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if is_offline {
                        AfterDisconnect::Block(ErrorStateCause::IsOffline)
                    } else {
                        AfterDisconnect::Reconnect(retry_attempt)
                    }
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(retry_attempt),
                Ok(TunnelCommand::Disconnect) | Err(_) => AfterDisconnect::Nothing,
                Ok(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
            },
        };

        EventConsequence::SameState(self)
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        let poll_result = match &mut self.exited {
            Some(exited) => exited.poll(),
            None => Ok(Async::Ready(None)),
        };

        match poll_result {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(block_reason)) => {
                NewState(self.after_disconnect(block_reason, shared_values))
            }
            Err(_) => NewState(self.after_disconnect(None, shared_values)),
        }
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
    type Bootstrap = (
        Option<CloseHandle>,
        Option<oneshot::Receiver<Option<ErrorStateCause>>>,
        AfterDisconnect,
    );

    fn enter(
        _: &mut SharedTunnelStateValues,
        (close_handle, exited, after_disconnect): Self::Bootstrap,
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
                exited,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands, shared_values)
            .or_else(Self::handle_exit_event, shared_values)
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
