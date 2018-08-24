use error_chain::ChainedError;
use futures::sync::mpsc;
use futures::Stream;

use super::{
    ConnectingState, Error, EventConsequence, SharedTunnelStateValues, TunnelCommand, TunnelState,
    TunnelStateWrapper,
};
use security::NetworkSecurity;

/// No tunnel is running.
pub struct DisconnectedState;

impl DisconnectedState {
    fn reset_security_policy(shared_values: &mut SharedTunnelStateValues) {
        debug!("Resetting security policy");
        if let Err(error) = shared_values.security.reset_policy() {
            let chained_error = Error::with_chain(error, "Failed to reset security policy");
            error!("{}", chained_error.display_chain());
        }
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = ();

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        _: Self::Bootstrap,
    ) -> TunnelStateWrapper {
        Self::reset_security_policy(shared_values);

        TunnelStateWrapper::from(DisconnectedState)
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
            Ok(_) => SameState(self),
            Err(_) => Finished,
        }
    }
}
