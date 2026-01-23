#[expect(clippy::allow_attributes)]
mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}
mod conversions;

pub use prost_types::{Duration, Timestamp};

pub use conversions::*;
pub use proto::*;
