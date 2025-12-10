//! This module defines wrapper types around [`Relay`], often to provide certain runtime guarantees
//! or disambiguate the type of relay which is used in the relay selector's internal APIs.

use mullvad_types::relay_list::Relay;

/// - [`WireguardConfig::Singlehop`]: A wireguard relay where VPN traffic enters and exits.
/// - [`WireguardConfig::Multihop`]: Two wireguard relays to be used in a multihop circuit. VPN
///   traffic will enter through `entry` and eventually exit through `exit` before the traffic will
///   actually be routed to the internet.
#[derive(Clone, Debug)]
pub enum WireguardConfig {
    /// An exit relay.
    Singlehop { exit: Relay },
    /// An entry and an exit relay.
    Multihop { exit: Relay, entry: Relay },
}

/// A type representing single Wireguard relay.
///
/// Before you can read any data out of a [`Singlehop`] value uou need to convert it to
/// [`WireguardConfig`]. This is easy since [`Singlehop`] implements [`Into<WireguardConfig>`].
///
/// # Why not simply use [`Relay`]?
/// The only way to construct a [`Singlehop`] value is with [`Singlehop::new`] which performs
/// additional validation which guarantees that the relay actually is a Wireguard relay, while
/// [`Relay`] is not guaranteed to be a Wireguard relay.
pub struct Singlehop(Relay);
/// A type representing two Wireguard relay - an entry and an exit.
///
/// Before you can read any data out of a [`Multihop`] value uou need to convert it to
/// [`WireguardConfig`]. This is easy since [`Multihop`] implements [`Into<WireguardConfig>`].
///
/// # Why not simply use [`Relay`]?
/// The same rationale as for [`Singlehop`] applies - [`Multihop::new`] performs additional
/// validation on the entry and exit relays.
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
        Self(exit)
    }
}

impl Multihop {
    pub const fn new(entry: Relay, exit: Relay) -> Self {
        Multihop { exit, entry }
    }
}
