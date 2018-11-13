use super::{
    BlockedState, ConnectingState, EventConsequence, ResultExt, SharedTunnelStateValues,
    TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::security::SecurityPolicy;
use error_chain::ChainedError;
use futures::sync::mpsc;
use futures::Stream;

/// No tunnel is running.
pub struct DisconnectedState;

impl DisconnectedState {
    fn set_security_policy(shared_values: &mut SharedTunnelStateValues) {
        let result = if shared_values.block_when_disconnected {
            let policy = SecurityPolicy::Blocked {
                allow_lan: shared_values.allow_lan,
            };
            shared_values
                .security
                .apply_policy(policy)
                .chain_err(|| "Failed to apply blocking security policy for disconnected state")
        } else {
            shared_values
                .security
                .reset_policy()
                .chain_err(|| "Failed to reset security policy")
        };
        if let Err(error) = result {
            log::error!("{}", error.display_chain());
        }
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = ();

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        _: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        Self::set_security_policy(shared_values);
        (
            TunnelStateWrapper::from(DisconnectedState),
            TunnelStateTransition::Disconnected,
        )
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                let changed = shared_values.allow_lan != allow_lan;
                shared_values.allow_lan = allow_lan;
                if changed {
                    Self::set_security_policy(shared_values);
                }
                SameState(self)
            }
            Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                let changed = shared_values.block_when_disconnected != block_when_disconnected;
                shared_values.block_when_disconnected = block_when_disconnected;
                if changed {
                    Self::set_security_policy(shared_values);
                }
                SameState(self)
            }
            Ok(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                SameState(self)
            }
            Ok(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Ok(TunnelCommand::Block(reason)) => {
                NewState(BlockedState::enter(shared_values, reason))
            }
            Ok(_) => SameState(self),
            Err(_) => Finished,
        }
    }
}
