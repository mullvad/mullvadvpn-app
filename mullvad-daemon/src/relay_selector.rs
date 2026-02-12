//! Relay selector gRPC service.

use mullvad_management_interface::types::relay_selector::*;
use mullvad_management_interface::{RelaySelectorService, Request, Response, Status};

/// The relay selector exposed as a gRPC service.
pub struct RelaySelectorServiceImpl;

impl RelaySelectorServiceImpl {
    pub fn new() -> Self {
        RelaySelectorServiceImpl {}
    }
}

#[mullvad_management_interface::async_trait]
impl RelaySelectorService for RelaySelectorServiceImpl {
    async fn partition_relays(
        &self,
        _: Request<Predicate>,
    ) -> Result<Response<RelayPartitions>, Status> {
        log::trace!("Handling `partition_relays` call with predicate: TODO");
        // TODO: Call out to the relay selector
        let partitions: RelayPartitions = RelayPartitions::default();
        Ok(Response::new(partitions))
    }
}
