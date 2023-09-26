#[allow(clippy::derive_partial_eq_without_eq)]
mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}
mod conversions;

pub use prost_types::{Duration, Timestamp};

pub use conversions::*;
pub use proto::*;
