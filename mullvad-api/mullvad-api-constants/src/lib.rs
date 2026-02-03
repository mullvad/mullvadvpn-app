use std::net::{IpAddr, Ipv4Addr};

pub mod env {
    pub const API_HOST_VAR: &str = "MULLVAD_API_HOST";
    pub const API_ADDR_VAR: &str = "MULLVAD_API_ADDR";
    pub const API_FORCE_DIRECT_VAR: &str = "MULLVAD_API_FORCE_DIRECT";
    pub const DISABLE_TLS_VAR: &str = "MULLVAD_API_DISABLE_TLS";
    pub const SIGSUM_TRUSTED_PUBKEYS_VAR: &str = "MULLVAD_SIGSUM_TRUSTED_PUBKEYS";
}

pub const API_HOST_DEFAULT: &str = "api.mullvad.net";
pub const API_IP_DEFAULT: IpAddr = IpAddr::V4(Ipv4Addr::new(45, 83, 223, 196));
pub const API_PORT_DEFAULT: u16 = 443;
pub const SIGSUM_TRUSTED_PUBKEYS_DEFAULT: &str = include_str!("trusted-sigsum-signing-pubkeys");
