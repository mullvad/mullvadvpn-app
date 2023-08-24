#![deny(rust_2018_idioms)]

pub mod account;
pub mod api_access_method;
pub mod auth_failed;
pub mod custom_list;
pub mod device;
pub mod endpoint;
pub mod location;
pub mod relay_constraints;
pub mod relay_list;
pub mod settings;
pub mod states;
pub mod version;
pub mod wireguard;

mod custom_tunnel;
pub use crate::custom_tunnel::*;

// b"mole" is [ 0x6d, 0x6f 0x6c, 0x65 ]
#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x6d6f6c65;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x6d6f6c65;
