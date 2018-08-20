use error_chain::ChainedError;
use futures::sync::mpsc;
use futures::Stream;

use talpid_core::firewall::Firewall;

use super::{
    ConnectingState, Error, EventConsequence, SharedTunnelStateValues, StateEntryResult,
    TunnelCommand, TunnelState, TunnelStateWrapper,
};

/// No tunnel is running.
pub struct DisconnectedState;

impl DisconnectedState {
    fn reset_security_policy(shared_values: &mut SharedTunnelStateValues) {
        debug!("Reset security policy");
        if let Err(error) = shared_values.firewall.reset_policy() {
            let chained_error = Error::with_chain(error, "Failed to reset security policy");
            error!("{}", chained_error.display_chain());
        }
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = ();

    fn enter(shared_values: &mut SharedTunnelStateValues, _: Self::Bootstrap) -> StateEntryResult {
        Self::reset_security_policy(shared_values);

        Ok(TunnelStateWrapper::from(DisconnectedState))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => {
                NewState(ConnectingState::enter(shared_values, parameters))
            }
            _ => SameState(self),
        }
    }
}
