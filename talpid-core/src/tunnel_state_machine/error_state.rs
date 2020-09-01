use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
use futures01::{sync::mpsc, Stream};
use talpid_types::{
    tunnel::{self as talpid_tunnel, ErrorStateCause, FirewallPolicyError},
    ErrorExt,
};

/// No tunnel is running and all network connections are blocked.
pub struct ErrorState {
    block_reason: ErrorStateCause,
}

impl ErrorState {
    /// Returns true if firewall policy was applied successfully
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), FirewallPolicyError> {
        let policy = FirewallPolicy::Blocked {
            allow_lan: shared_values.allow_lan,
        };

        shared_values
            .firewall
            .apply_policy(policy)
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for blocked state"
                    )
                );
                match error {
                    #[cfg(windows)]
                    crate::firewall::Error::ApplyingBlockedPolicy(policy_error) => policy_error,
                    _ => FirewallPolicyError::Generic,
                }
            })
    }

    /// Returns true if a new tunnel device was successfully created.
    #[cfg(target_os = "android")]
    fn create_blocking_tun(shared_values: &mut SharedTunnelStateValues) -> bool {
        match shared_values.tun_provider.create_tun_if_closed() {
            Ok(()) => true,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to open tunnel adapter to drop packets for blocked state"
                    )
                );
                false
            }
        }
    }
}

impl TunnelState for ErrorState {
    type Bootstrap = ErrorStateCause;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        block_reason: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        #[cfg(not(target_os = "android"))]
        let block_failure = Self::set_firewall_policy(shared_values).err();
        #[cfg(target_os = "android")]
        let block_failure = if !Self::create_blocking_tun(shared_values) {
            Some(FirewallPolicyError::Generic)
        } else {
            None
        };
        (
            TunnelStateWrapper::from(ErrorState {
                block_reason: block_reason.clone(),
            }),
            TunnelStateTransition::Error(talpid_tunnel::ErrorState::new(
                block_reason,
                block_failure,
            )),
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
                if let Err(error_state_cause) = shared_values.set_allow_lan(allow_lan) {
                    NewState(Self::enter(shared_values, error_state_cause))
                } else {
                    let _ = Self::set_firewall_policy(shared_values);
                    SameState(self)
                }
            }
            Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self)
            }
            Ok(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if !is_offline && self.block_reason == ErrorStateCause::IsOffline {
                    NewState(ConnectingState::enter(shared_values, 0))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectedState::enter(shared_values, true))
            }
            Ok(TunnelCommand::Block(reason)) => NewState(ErrorState::enter(shared_values, reason)),
        }
    }
}
