use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
#[cfg(windows)]
use crate::split_tunnel;
use futures::StreamExt;
#[cfg(windows)]
use std::ffi::OsStr;
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
            allowed_endpoint: shared_values.allowed_endpoint.clone(),
        };

        #[cfg(target_os = "linux")]
        shared_values.disable_connectivity_check();

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
        match shared_values.tun_provider.create_blocking_tun() {
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

    #[cfg(windows)]
    fn apply_split_tunnel_config<T: AsRef<OsStr>>(
        shared_values: &SharedTunnelStateValues,
        paths: &[T],
    ) -> Result<(), split_tunnel::Error> {
        let split_tunnel = shared_values
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex");
        split_tunnel.set_paths(paths)
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
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match runtime.block_on(commands.next()) {
            Some(TunnelCommand::AllowLan(allow_lan)) => {
                if let Err(error_state_cause) = shared_values.set_allow_lan(allow_lan) {
                    NewState(Self::enter(shared_values, error_state_cause))
                } else {
                    let _ = Self::set_firewall_policy(shared_values);
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.set_allowed_endpoint(endpoint) {
                    let _ = Self::set_firewall_policy(shared_values);

                    #[cfg(target_os = "android")]
                    if !Self::create_blocking_tun(shared_values) {
                        return NewState(Self::enter(
                            shared_values,
                            ErrorStateCause::SetFirewallPolicyError(FirewallPolicyError::Generic),
                        ));
                    }
                }
                if let Err(_) = tx.send(()) {
                    log::error!("The AllowEndpoint receiver was dropped");
                }
                SameState(self.into())
            }
            Some(TunnelCommand::Dns(servers)) => {
                if let Err(error_state_cause) = shared_values.set_dns_servers(servers) {
                    NewState(Self::enter(shared_values, error_state_cause))
                } else {
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self.into())
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if !is_offline && self.block_reason == ErrorStateCause::IsOffline {
                    NewState(ConnectingState::enter(shared_values, 0))
                } else {
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Some(TunnelCommand::Disconnect) | None => {
                #[cfg(target_os = "linux")]
                shared_values.reset_connectivity_check();
                NewState(DisconnectedState::enter(shared_values, true))
            }
            Some(TunnelCommand::Block(reason)) => {
                NewState(ErrorState::enter(shared_values, reason))
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
                SameState(self.into())
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                // TODO: Do nothing here?
                let _ = result_tx.send(Self::apply_split_tunnel_config(shared_values, &paths));
                SameState(self.into())
            }
        }
    }
}
