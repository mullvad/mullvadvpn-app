use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::{Ipv4Addr, Ipv6Addr};

/// The networks that the unmodified Mullvad app treats as local. See [`ALLOWED_LAN_NETS`] for
/// the list that is actually used, which also includes the networks from `extra-lan-networks.txt`.
pub const BASE_LAN_NETS: [IpNetwork; 6] = [
    v4(Ipv4Addr::new(10, 0, 0, 0), 8),
    v4(Ipv4Addr::new(172, 16, 0, 0), 12),
    v4(Ipv4Addr::new(192, 168, 0, 0), 16),
    v4(Ipv4Addr::new(169, 254, 0, 0), 16),
    v6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10),
    v6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),
];

// Defines `EXTRA_LAN_NETS`, generated from `extra-lan-networks.txt` in the repository root by
// `cargo run -p talpid-types --bin generate-extra-lan-nets`. See `extra_lan_config.rs`.
include!("extra_lan_nets.rs");

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
///
/// This is [`BASE_LAN_NETS`] followed by [`EXTRA_LAN_NETS`].
pub const ALLOWED_LAN_NETS: [IpNetwork; BASE_LAN_NETS.len() + EXTRA_LAN_NETS.len()] =
    concat_lan_nets();

const fn concat_lan_nets() -> [IpNetwork; BASE_LAN_NETS.len() + EXTRA_LAN_NETS.len()] {
    let mut out = [BASE_LAN_NETS[0]; BASE_LAN_NETS.len() + EXTRA_LAN_NETS.len()];
    let mut i = 0;
    while i < BASE_LAN_NETS.len() {
        out[i] = BASE_LAN_NETS[i];
        i += 1;
    }
    let mut j = 0;
    while j < EXTRA_LAN_NETS.len() {
        out[i + j] = EXTRA_LAN_NETS[j];
        j += 1;
    }
    out
}

/// When "allow local network" is enabled the app will allow traffic to these networks.
pub const ALLOWED_LAN_MULTICAST_NETS: [IpNetwork; 8] = [
    // Local network broadcast. Not routable
    v4(Ipv4Addr::new(255, 255, 255, 255), 32),
    // Local subnetwork multicast. Not routable
    v4(Ipv4Addr::new(224, 0, 0, 0), 24),
    // Admin-local IPv4 multicast.
    v4(Ipv4Addr::new(239, 0, 0, 0), 8),
    // Interface-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16),
    // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
    v6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16),
    // Realm-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16),
    // Admin-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16),
    // Site-local IPv6 multicast.
    v6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16),
];

// Short-hand for `IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())`.
const fn v4(address: Ipv4Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())
}

// Short-hand for `IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())`.
const fn v6(address: Ipv6Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The full list must be the base list followed by the extra networks, in order.
    #[test]
    fn allowed_lan_nets_is_base_then_extra() {
        let expected: Vec<IpNetwork> = BASE_LAN_NETS
            .iter()
            .chain(EXTRA_LAN_NETS.iter())
            .copied()
            .collect();
        assert_eq!(ALLOWED_LAN_NETS.to_vec(), expected);
    }

    /// The generated `extra_lan_nets.rs` must match `extra-lan-networks.txt`. If this fails,
    /// run `cargo run -p talpid-types --bin generate-extra-lan-nets` and commit the result.
    #[test]
    fn extra_lan_nets_match_file() {
        use crate::net::extra_lan_config::{EXTRA_LAN_NETWORKS_FILE, is_permitted, parse};

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../extra-lan-networks.txt");
        let contents = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("{EXTRA_LAN_NETWORKS_FILE} must exist: {err}"));
        let expected = parse(&contents).unwrap_or_else(|err| panic!("{path}: {err}"));

        assert_eq!(
            EXTRA_LAN_NETS.to_vec(),
            expected,
            "extra_lan_nets.rs is out of date: run \
             `cargo run -p talpid-types --bin generate-extra-lan-nets`"
        );
        for net in EXTRA_LAN_NETS {
            assert!(
                is_permitted(net),
                "{net} is outside the permitted address space"
            );
            assert!(ALLOWED_LAN_NETS.contains(&net));
        }
    }
}
