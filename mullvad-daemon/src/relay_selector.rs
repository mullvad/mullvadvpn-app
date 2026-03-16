//! Relay selector gRPC service.

use mullvad_management_interface::types::relay_selector as proto;
use mullvad_management_interface::{RelaySelectorService, Request, Response, Status};
use mullvad_relay_selector::RelaySelector;
use mullvad_types::relay_selector::Predicate;

/// The relay selector exposed as a gRPC service. See `relay_selector.proto` for API.
pub struct RelaySelectorServiceImpl(RelaySelector);

impl RelaySelectorServiceImpl {
    pub fn new(relay_selector: RelaySelector) -> Self {
        RelaySelectorServiceImpl(relay_selector)
    }
}

#[mullvad_management_interface::async_trait]
impl RelaySelectorService for RelaySelectorServiceImpl {
    async fn partition_relays(
        &self,
        predicate: Request<proto::Predicate>,
    ) -> Result<Response<proto::RelayPartitions>, Status> {
        let predicate = predicate.into_inner();
        let predicate = Predicate::try_from(predicate)?;
        let partitions = self.0.partition_relays(predicate);
        let partitions = proto::RelayPartitions::from(partitions);
        Ok(Response::new(partitions))
    }
}
