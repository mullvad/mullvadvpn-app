// This module includes both proto packages in a nested structure
// so they can reference each other properly.
#[expect(clippy::allow_attributes)]
mod proto {
    // Include the management_interface proto first since relay_selector depends on it
    pub mod management_interface {
        tonic::include_proto!("mullvad_daemon.management_interface");
    }

    pub mod relay_selector {
        tonic::include_proto!("mullvad_daemon.relay_selector");
    }
}

// Re-export the relay selector service types
pub use proto::relay_selector::relay_selector_service_client;
pub use proto::relay_selector::relay_selector_service_server;

// Re-export the main relay selector message types
pub use proto::relay_selector::{
    Context, DiscardedRelay, EntryConstraints, EntryRelayConstraints, Hostname, MatchingRelays,
    MultiHopConstraints, RelayConstraints, RelayFilterReasons,
};

// Re-export the client and server types for convenience
pub type RelaySelectorServiceClient =
    relay_selector_service_client::RelaySelectorServiceClient<crate::Channel>;
pub use relay_selector_service_server::{RelaySelectorService, RelaySelectorServiceServer};
