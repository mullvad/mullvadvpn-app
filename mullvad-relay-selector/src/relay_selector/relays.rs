//! TODO: Document this module

use mullvad_types::relay_list::{Relay, RelayEndpointData};

// TODO: Import `Either` and convert `Multihop` and `Singlehop` into concrete types.
/// This struct defines the different Wireguard relays the the relay selector can end up selecting
/// for an arbitrary Wireguard [`query`].
///
/// - [`WireguardConfig::Singlehop`]; A normal wireguard relay where VPN traffic enters and exits
///   through this sole relay.
/// - [`WireguardConfig::Multihop`]; Two wireguard relays to be used in a multihop circuit. VPN
///   traffic will enter through `entry` and eventually come out from `exit` before the traffic will
///   actually be routed to the broader internet.
#[derive(Clone, Debug)]
pub enum WireguardConfig {
    /// Strongly prefer to instantiate this variant using [`WireguardConfig::singlehop`] as that
    /// will assert that the relay is of the expected type.
    Singlehop { exit: Relay },
    /// Strongly prefer to instantiate this variant using [`WireguardConfig::multihop`] as that
    /// will assert that the entry & exit relays are of the expected type.
    Multihop { exit: Relay, entry: Relay },
}

/// TODO: Document
pub struct Singlehop(Relay);
/// TODO: Document
pub struct Multihop {
    entry: Relay,
    exit: Relay,
}

impl From<Singlehop> for WireguardConfig {
    fn from(relay: Singlehop) -> Self {
        Self::Singlehop { exit: relay.0 }
    }
}

impl From<Multihop> for WireguardConfig {
    fn from(relay: Multihop) -> Self {
        WireguardConfig::Multihop {
            exit: relay.exit,
            entry: relay.entry,
        }
    }
}

impl Singlehop {
    pub const fn new(exit: Relay) -> Self {
        // FIXME: This assert would be better to encode at the type level.
        assert!(matches!(
            exit.endpoint_data,
            RelayEndpointData::Wireguard(_)
        ));
        Self(exit)
    }
}

impl Multihop {
    pub const fn new(entry: Relay, exit: Relay) -> Self {
        // FIXME: This assert would be better to encode at the type level.
        assert!(matches!(
            exit.endpoint_data,
            RelayEndpointData::Wireguard(_)
        ));
        // FIXME: This assert would be better to encode at the type level.
        assert!(matches!(
            entry.endpoint_data,
            RelayEndpointData::Wireguard(_)
        ));
        Multihop { exit, entry }
    }
}
