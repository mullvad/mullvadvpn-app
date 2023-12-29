use crate::net::{Endpoint, GenericTunnelOptions};
use serde::{Deserialize, Serialize};

use super::proxy::CustomProxy;

/// Information needed by `OpenVpnMonitor` to establish a tunnel connection.
/// See [`crate::net::TunnelParameters`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct TunnelParameters {
    pub config: ConnectionConfig,
    pub options: TunnelOptions,
    pub generic_options: GenericTunnelOptions,
    pub proxy: Option<CustomProxy>,
    #[cfg(target_os = "linux")]
    pub fwmark: u32,
}

/// Connection configuration used by [`TunnelParameters`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub endpoint: Endpoint,
    pub username: String,
    pub password: String,
}

impl ConnectionConfig {
    pub fn new(endpoint: Endpoint, username: String, password: String) -> ConnectionConfig {
        Self {
            endpoint,
            username,
            password,
        }
    }
}

/// `TunnelOptions` contains options for an OpenVPN tunnel that should be applied
/// irrespective of the relay parameters - i.e. have nothing to do with the particular
/// OpenVPN server, but do affect the connection.
/// Stored in [`TunnelParameters`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TunnelOptions {
    /// Optional argument for openvpn to try and limit TCP packet size,
    /// as discussed [here](https://openvpn.net/archive/openvpn-users/2003-11/msg00154.html)
    pub mssfix: Option<u16>,
}
