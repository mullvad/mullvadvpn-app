//! Relay selector gRPC service.

use mullvad_management_interface::types::relay_selector as proto;
use mullvad_management_interface::{RelaySelectorService, Request, Response, Status};
use mullvad_relay_selector::CustomListProvider;
use mullvad_types::relay_selector::Predicate;

use crate::relay_selector::RelaySelectorIO;

/// The relay selector exposed as a gRPC service. See `relay_selector.proto` for API.
pub struct RelaySelectorServiceImpl(RelaySelectorIO);

impl RelaySelectorServiceImpl {
    pub fn new(relay_selector: RelaySelectorIO) -> Self {
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
        let predicate = predicate.into_domain(&self.0.config.custom_lists())?;
        let partitions = self.0.partition_relays(predicate);
        let partitions = proto::RelayPartitions::from(partitions);
        Ok(Response::new(partitions))
    }
}
