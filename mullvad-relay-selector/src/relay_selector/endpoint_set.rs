//! Types representing the set of valid connection endpoints for a relay.
//!
//! A [`RelayEndpointSet`] captures all the ways a client can connect to a specific relay:
//! plain WireGuard, Shadowsocks, UDP2TCP, QUIC, and LWO. It is built once per relay when
//! the relay list is loaded and used in two ways:
//!
//! - **Filtering** ([`RelayEndpointSet::obfuscation_verdict`]): determines whether a relay
//!   satisfies a given set of entry constraints.
//! - **Selection** ([`RelayEndpointSet::get_wireguard_obfuscator`]): picks a concrete
//!   endpoint address and, if applicable, an obfuscator config for an accepted relay.
//!
//! IP version filtering is applied at selection time, not at construction time, since IP
//! availability is a runtime property that can change between connection attempts.

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    ops::RangeInclusive,
};

use rand::{
    Rng,
    seq::{IndexedRandom, IteratorRandom},
};

use mullvad_types::{
    constraints::Constraint,
    relay_list::{EndpointData, Quic, WireguardRelay},
    relay_selector::{EntrySpecificConstraints, Reason},
};
use talpid_types::net::{
    IpVersion,
    obfuscation::{ObfuscatorConfig, Obfuscators},
};
use vec1::Vec1;

use crate::query::ObfuscationMode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Found no valid port matching the selected settings")]
    NoMatchingPort,
    #[error("Found no valid IP protocol matching the selected settings")]
    NoMatchingAddresses,
    #[error("The selected relay does not support the selected obfuscation method")]
    MissingSupport,
}

// ---------------------------------------------------------------------------
// Core endpoint set types
// ---------------------------------------------------------------------------

/// All connection endpoints available on a specific relay.
///
/// This is a capability map: it describes what the relay supports, not what the user has
/// requested. The user's obfuscation and IP version preferences are applied at selection time.
#[derive(Debug, Clone)]
pub struct RelayEndpointSet {
    /// The relay's WireGuard endpoints. Always available for an active relay.
    wireguard: WireguardEndpoints,
    /// Shadowsocks obfuscation endpoints, if the relay has dedicated Shadowsocks IPs
    /// or Shadowsocks port ranges are available in the relay list.
    shadowsocks: Option<ShadowsocksEndpoints>,
    /// Available UDP2TCP ports, if UDP2TCP is supported. Uses the WireGuard endpoint addresses.
    udp2tcp_ports: Option<Vec1<u16>>,
    /// QUIC obfuscation endpoints, if the relay supports QUIC.
    quic: Option<QuicEndpoints>,
    /// Whether the relay supports LWO. LWO reuses the WireGuard endpoint,
    /// so no additional address information is needed.
    lwo: bool,
}

/// The relay's WireGuard endpoint addresses and valid port ranges.
///
/// Every relay has at least an IPv4 address. IPv6 may or may not be available.
/// Port ranges come from the relay list's global [`EndpointData`].
#[derive(Debug, Clone)]
struct WireguardEndpoints {
    ipv4: Ipv4Addr,
    ipv6: Option<Ipv6Addr>,
    port_ranges: Vec1<RangeInclusive<u16>>,
}

/// Shadowsocks obfuscation endpoints.
///
/// Shadowsocks traffic can be routed through either:
/// - **Dedicated IPs**: Addresses specifically allocated for Shadowsocks on this relay.
///   All ports are valid on these addresses.
/// - **WireGuard IP fallback**: The relay's standard WireGuard address, but only on the
///   port ranges designated for Shadowsocks in the relay list.
///
/// Both paths may be available simultaneously. At selection time, dedicated IPs are
/// generally preferred (broader port range, potentially better IP version coverage).
#[derive(Debug, Clone)]
struct ShadowsocksEndpoints {
    /// Dedicated Shadowsocks addresses (all ports valid).
    /// May contain a mix of IPv4 and IPv6 addresses.
    dedicated_addrs: HashSet<IpAddr>,
    /// Port ranges for Shadowsocks over the WireGuard endpoint. `None` when no such port
    /// ranges exist (in which case only dedicated addresses may be used).
    shadowsocks_port_ranges: Option<Vec1<RangeInclusive<u16>>>,
}

/// QUIC (MASQUE proxy) obfuscation endpoints.
///
/// QUIC has its own set of addresses, separate from the relay's WireGuard endpoint.
/// The port is fixed at 443.
#[derive(Debug, Clone)]
struct QuicEndpoints {
    ipv4: Vec<Ipv4Addr>,
    ipv6: Vec<Ipv6Addr>,
    hostname: String,
    auth_token: String,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl RelayEndpointSet {
    /// Build a [`RelayEndpointSet`] from a relay and the global endpoint data.
    /// Returns `None` if the wireguard port ranges are empty.
    ///
    /// This captures all connection methods the relay supports. No filtering
    /// by user preferences or IP version is applied here.
    pub fn new(relay: &WireguardRelay, endpoint_data: &EndpointData) -> Option<Self> {
        // WireGuard port ranges must be non-empty for any connection to work.
        let wg_port_ranges = Vec1::try_from_vec(endpoint_data.port_ranges.clone()).ok()?;

        let wireguard = WireguardEndpoints {
            ipv4: relay.ipv4_addr_in,
            ipv6: relay.ipv6_addr_in,
            port_ranges: wg_port_ranges,
        };

        let shadowsocks = ShadowsocksEndpoints::new(relay, endpoint_data);
        let udp2tcp_ports = Vec1::try_from_vec(endpoint_data.udp2tcp_ports.clone()).ok();
        let quic = QuicEndpoints::new(relay);
        let lwo = relay.endpoint_data.lwo;

        Some(RelayEndpointSet {
            wireguard,
            shadowsocks,
            udp2tcp_ports,
            quic,
            lwo,
        })
    }

    pub(crate) fn obfuscation_verdict(
        &self,
        EntrySpecificConstraints {
            obfuscation,
            ip_version,
            ..
        }: &EntrySpecificConstraints,
    ) -> Verdict {
        match obfuscation {
            // Constraint::Any means "auto" — we can always fallback to routing traffic through the plain WireGuard endpoint.
            Constraint::Any | Constraint::Only(ObfuscationMode::Off) => self
                .wireguard
                .supports_ip_version(*ip_version)
                .if_false(Reason::IpVersion),

            Constraint::Only(ObfuscationMode::Port(port)) => Verdict::all([
                self.wireguard
                    .supports_port(port.get())
                    .if_false(Reason::Port),
                self.wireguard
                    .supports_ip_version(*ip_version)
                    .if_false(Reason::IpVersion),
            ]),

            Constraint::Only(ObfuscationMode::Udp2tcp(_)) => match self.udp2tcp_ports {
                None => Verdict::reject(Reason::Obfuscation),
                Some(_) => self
                    .wireguard
                    .supports_ip_version(*ip_version)
                    .if_false(Reason::IpVersion),
            },

            Constraint::Only(ObfuscationMode::Shadowsocks(settings)) => match &self.shadowsocks {
                None => Verdict::reject(Reason::Obfuscation),
                Some(ss) => ss.verdict(&self.wireguard, *ip_version, settings.port),
            },

            Constraint::Only(ObfuscationMode::Quic) => match &self.quic {
                None => Verdict::reject(Reason::Obfuscation),
                // QuicEndpoints is only Some when the relay has at least one QUIC
                // address, so the "no addresses at all" case is impossible. We only
                // need to distinguish: IPs match the version vs. IPs exist but are
                // the wrong version.
                Some(quic) => quic
                    .supports_ip_version(*ip_version)
                    .if_false(Reason::IpVersion),
            },

            // LWO reuses the WireGuard endpoint, so IP version availability is the same as
            // for plain WireGuard.
            Constraint::Only(ObfuscationMode::Lwo(_)) => Verdict::all([
                self.lwo.if_false(Reason::Obfuscation),
                self.wireguard
                    .supports_ip_version(*ip_version)
                    .if_false(Reason::IpVersion),
            ]),
        }
    }

    pub fn get_wireguard_obfuscator(
        &self,
        query: &Constraint<ObfuscationMode>,
        ip_version: Constraint<IpVersion>,
    ) -> Result<(SocketAddr, Option<Obfuscators>), Error> {
        let port = if let Constraint::Only(ObfuscationMode::Port(port)) = query {
            port.get()
        } else {
            Constraint::Any
        };
        let wireguard_endpoint = self
            .wireguard
            .random_endpoint(ip_version, port)
            .ok_or(Error::NoMatchingAddresses)?;

        let mode = match query {
            Constraint::Only(mode) => mode,
            #[cfg(not(feature = "staggered-obfuscation"))]
            Constraint::Any => return Ok((wireguard_endpoint, None)),
            #[cfg(feature = "staggered-obfuscation")]
            Constraint::Any => {
                let staggered_obfuscator = self
                    .staggered_obfuscator(wireguard_endpoint, ip_version)
                    .ok_or(Error::MissingSupport)?;
                return Ok((wireguard_endpoint, Some(staggered_obfuscator)));
            }
        };

        let obfuscator_config = match mode {
            ObfuscationMode::Off | ObfuscationMode::Port(_) => None,
            ObfuscationMode::Udp2tcp(settings) => Some(Obfuscators::Single(
                self.udp2tcp_config(wireguard_endpoint.ip(), settings.port)?,
            )),
            ObfuscationMode::Shadowsocks(settings) => Some(Obfuscators::Single(
                self.shadowsocks_config(ip_version, settings.port)?,
            )),
            ObfuscationMode::Quic => Some(Obfuscators::Single(self.quic_config(ip_version)?)),
            ObfuscationMode::Lwo(settings) => Some(Obfuscators::Single(
                self.lwo_config(wireguard_endpoint.ip(), settings.port)?,
            )),
        };
        Ok((wireguard_endpoint, obfuscator_config))
    }

    /// Build a multiplexer [`Obfuscators`] config that tries all available obfuscation methods
    /// in parallel alongside the plain `direct_endpoint`.
    #[cfg(feature = "staggered-obfuscation")]
    fn staggered_obfuscator(
        &self,
        direct_endpoint: SocketAddr,
        ip_version: Constraint<IpVersion>,
    ) -> Option<Obfuscators> {
        let configs: Vec<ObfuscatorConfig> = [
            self.lwo_config(direct_endpoint.ip(), Constraint::Any),
            self.udp2tcp_config(direct_endpoint.ip(), Constraint::Any),
            self.shadowsocks_config(ip_version, Constraint::Any),
            self.quic_config(ip_version),
        ]
        .into_iter()
        .flatten()
        .collect();

        Obfuscators::multiplexer(Some(direct_endpoint), &configs)
    }

    /// Build a Udp2Tcp obfuscator config, or return `None` if udp2tcp is unsupported or no
    /// udp2tcp port satisfies `port`. Udp2Tcp modifies WG traffic in place, so `wg_ip` must
    /// be the chosen WG endpoint IP.
    fn udp2tcp_config(
        &self,
        wg_ip: IpAddr,
        port: Constraint<u16>,
    ) -> Result<ObfuscatorConfig, Error> {
        let ports = self.udp2tcp_ports.as_ref().ok_or(Error::MissingSupport)?;
        let port = match port {
            Constraint::Only(p) => ports.contains(&p).then_some(p),
            Constraint::Any => ports.choose(&mut rand::rng()).copied(),
        }
        .ok_or(Error::NoMatchingPort)?;
        Ok(ObfuscatorConfig::Udp2Tcp {
            endpoint: SocketAddr::new(wg_ip, port),
        })
    }

    /// Build an LWO obfuscator config, or return `None` if LWO is unsupported or no port in
    /// the WG port ranges satisfies `port`. LWO wraps the WG socket on the relay side, so
    /// `wg_ip` must be the chosen WG endpoint IP.
    fn lwo_config(&self, wg_ip: IpAddr, port: Constraint<u16>) -> Result<ObfuscatorConfig, Error> {
        if !self.lwo {
            return Err(Error::MissingSupport);
        }
        let port = random_port_in_ranges(&self.wireguard.port_ranges, port)
            .ok_or(Error::NoMatchingPort)?;
        Ok(ObfuscatorConfig::Lwo {
            endpoint: SocketAddr::new(wg_ip, port),
        })
    }

    /// Build a Shadowsocks obfuscator config, or return `None` if unsupported or no
    /// endpoint satisfies the constraints.
    fn shadowsocks_config(
        &self,
        ip_version: Constraint<IpVersion>,
        port: Constraint<u16>,
    ) -> Result<ObfuscatorConfig, Error> {
        let ss = self.shadowsocks.as_ref().ok_or(Error::MissingSupport)?;
        let endpoint = ss
            .random_endpoint(&self.wireguard, ip_version, port)
            .ok_or(Error::NoMatchingAddresses)?;
        Ok(ObfuscatorConfig::Shadowsocks { endpoint })
    }

    /// Build a QUIC obfuscator config, or return `None` if unsupported or no endpoint
    /// matches `ip_version`.
    fn quic_config(&self, ip_version: Constraint<IpVersion>) -> Result<ObfuscatorConfig, Error> {
        let quic = self.quic.as_ref().ok_or(Error::MissingSupport)?;
        let endpoint = quic
            .random_endpoint(ip_version)
            .ok_or(Error::NoMatchingAddresses)?;
        Ok(ObfuscatorConfig::Quic {
            hostname: quic.hostname.clone(),
            endpoint,
            auth_token: quic.auth_token.clone(),
        })
    }
}

impl WireguardEndpoints {
    /// Whether the WireGuard endpoint supports the given IP version.
    fn supports_ip_version(&self, ip_version: Constraint<IpVersion>) -> bool {
        self.select_ip(ip_version).is_some()
    }

    /// Whether the WireGuard endpoint supports the given port.
    fn supports_port(&self, port: Constraint<u16>) -> bool {
        port_in_ranges(&self.port_ranges, port)
    }

    fn random_endpoint(
        &self,
        ip_version: Constraint<IpVersion>,
        port: Constraint<u16>,
    ) -> Option<SocketAddr> {
        let ip = self.select_ip(ip_version)?;
        let port = random_port_in_ranges(&self.port_ranges, port)?;
        Some(SocketAddr::new(ip, port))
    }

    /// Select an address for the requested IP version, if available.
    /// IPv4 is always available. IPv6 depends on the relay.
    fn select_ip(&self, ip_version: Constraint<IpVersion>) -> Option<IpAddr> {
        match ip_version {
            Constraint::Any | Constraint::Only(IpVersion::V4) => Some(self.ipv4.into()),
            Constraint::Only(IpVersion::V6) => self.ipv6.map(|ip| ip.into()),
        }
    }
}

impl ShadowsocksEndpoints {
    fn new(relay: &WireguardRelay, endpoint_data: &EndpointData) -> Option<Self> {
        let dedicated_addrs = relay.endpoint_data.shadowsocks_extra_addr_in.clone();
        let ss_port_ranges = endpoint_data.shadowsocks_port_ranges.clone();

        // Shadowsocks is available if there are dedicated addresses OR WireGuard port ranges.
        if dedicated_addrs.is_empty() && ss_port_ranges.is_empty() {
            return None;
        }

        // The WG fallback is only available when there are Shadowsocks-designated port ranges.
        // Vec1 guarantees non-emptiness, so port presence is encoded in the Option itself.
        let shadowsocks_port_ranges = Vec1::try_from_vec(ss_port_ranges).ok();

        Some(ShadowsocksEndpoints {
            dedicated_addrs,
            shadowsocks_port_ranges,
        })
    }

    /// Returns whether there are any dedicated Shadowsocks addresses matching `ip_version`.
    fn has_dedicated_addr(&self, ip_version: Constraint<IpVersion>) -> bool {
        match ip_version {
            Constraint::Any => !self.dedicated_addrs.is_empty(),
            Constraint::Only(ip_version) => self
                .dedicated_addrs
                .iter()
                .any(|ip| IpVersion::from(*ip) == ip_version),
        }
    }

    /// Pick a random Shadowsocks endpoint for the given IP version and port preference.
    ///
    /// Dedicated Shadowsocks addresses are preferred (all ports are valid on them).
    /// Falls back to the WireGuard address with Shadowsocks-designated port ranges.
    ///
    /// Returns `None` if neither dedicated addresses nor the WireGuard fallback
    /// can satisfy the requested IP version and port.
    fn random_endpoint(
        &self,
        wg: &WireguardEndpoints,
        ip_version: Constraint<IpVersion>,
        port: Constraint<u16>,
    ) -> Option<SocketAddr> {
        // Try dedicated addresses first (any port is valid).
        if let Some(ip) = self
            .dedicated_addrs
            .iter()
            .filter(|&&ip| match ip_version {
                Constraint::Any => true,
                Constraint::Only(ip_version) => IpVersion::from(ip) == ip_version,
            })
            .choose(&mut rand::rng())
        {
            let port = port.unwrap_or_else(|| rand::rng().random_range(1u16..=u16::MAX));
            return Some(SocketAddr::new(*ip, port));
        }

        // Fall back to the WireGuard address with restricted Shadowsocks port ranges.
        let addr = wg.select_ip(ip_version)?;
        let port = random_port_in_ranges(self.shadowsocks_port_ranges.as_ref()?, port)?;
        Some(SocketAddr::new(addr, port))
    }

    /// Full Shadowsocks verdict: tries dedicated addresses first, then falls back to the
    /// WireGuard endpoint.
    ///
    /// Dedicated addresses have no port restriction (any port is valid on them), so they take
    /// priority over the WireGuard fallback. If dedicated addresses exist but are the wrong IP
    /// version, `Reason::IpVersion` is reported regardless of what the WG path says, because
    /// switching IP version would unlock those dedicated addresses.
    fn verdict(
        &self,
        wg: &WireguardEndpoints,
        ip_version: Constraint<IpVersion>,
        port: Constraint<u16>,
    ) -> Verdict {
        // Dedicated addresses accept any port; prefer them.
        if self.has_dedicated_addr(ip_version) {
            return Verdict::Accept;
        }

        // Fall back to the WG path, which only works when SS-designated port ranges exist.
        let wg_verdict = match &self.shadowsocks_port_ranges {
            None => Verdict::reject(Reason::Obfuscation),
            Some(port_ranges) => Verdict::all([
                port_in_ranges(port_ranges, port).if_false(Reason::Port),
                wg.supports_ip_version(ip_version)
                    .if_false(Reason::IpVersion),
            ]),
        };

        // If the WG path rejects but dedicated addrs exist, they must be the wrong IP family
        // (has_dedicated_addr was checked above) — switching IP version is the most
        // actionable fix, so override the WG rejection reasons. Note: it's possible the WG
        // path could be unblocked by a port change too, but the reject-reasons model doesn't
        // let us express "either of these would work".
        if matches!(wg_verdict, Verdict::Reject(_)) && !self.dedicated_addrs.is_empty() {
            return Verdict::reject(Reason::IpVersion);
        }
        wg_verdict
    }
}

impl QuicEndpoints {
    fn new(relay: &WireguardRelay) -> Option<Self> {
        let quic: &Quic = relay.endpoint_data.quic.as_ref()?;
        let ipv4: Vec<Ipv4Addr> = quic.in_ipv4().collect();
        let ipv6: Vec<Ipv6Addr> = quic.in_ipv6().collect();

        // QUIC requires at least one address of any version.
        if ipv4.is_empty() && ipv6.is_empty() {
            return None;
        }

        Some(QuicEndpoints {
            ipv4,
            ipv6,
            hostname: quic.hostname().to_owned(),
            auth_token: quic.auth_token().to_owned(),
        })
    }

    /// Whether the QUIC endpoint has addresses for the given IP version.
    fn supports_ip_version(&self, ip_version: Constraint<IpVersion>) -> bool {
        ip_version.is_any_or(|ip_version| match ip_version {
            IpVersion::V4 => !self.ipv4.is_empty(),
            IpVersion::V6 => !self.ipv6.is_empty(),
        })
    }

    /// Pick a random QUIC endpoint for the given IP version.
    ///
    /// `Constraint::Any` prefers IPv4 but falls back to IPv6 if the relay has no IPv4 QUIC
    /// addresses. Returns `None` if no QUIC addresses match the requested IP version.
    fn random_endpoint(&self, ip_version: Constraint<IpVersion>) -> Option<SocketAddr> {
        let ipv4 = self.ipv4.iter().copied().map(IpAddr::V4);
        let ipv6 = self.ipv6.iter().copied().map(IpAddr::V6);

        let ip = match ip_version {
            Constraint::Only(IpVersion::V4) => ipv4.choose(&mut rand::rng()),
            Constraint::Only(IpVersion::V6) => ipv6.choose(&mut rand::rng()),
            // Prefer IPv4, but fall back to IPv6 if the relay has no IPv4 QUIC addresses.
            Constraint::Any => ipv4
                .choose(&mut rand::rng())
                .or_else(|| ipv6.choose(&mut rand::rng())),
        }?;
        Some(SocketAddr::from((ip, Quic::port())))
    }
}

/// Pick a port from a list of ranges, respecting a constraint.
fn random_port_in_ranges(ranges: &[RangeInclusive<u16>], port: Constraint<u16>) -> Option<u16> {
    match port {
        Constraint::Only(p) => ranges.iter().any(|r| r.contains(&p)).then_some(p),
        Constraint::Any => ranges.iter().cloned().flatten().choose(&mut rand::rng()),
    }
}

/// Whether `port` falls within any of the given ranges (or is unconstrained).
fn port_in_ranges(ranges: &[RangeInclusive<u16>], port: Constraint<u16>) -> bool {
    port.is_any_or(|p| ranges.iter().any(|r| r.contains(&p)))
}

// ---------------------------------------------------------------------------
// Verdict
// ---------------------------------------------------------------------------

/// Whether a relay satisfies the requested constraints.
///
/// If rejected, all [reasons](Reason) for that judgment are provided.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Accept,
    Reject(Vec<Reason>),
}

impl Verdict {
    /// Accept if both verdicts are accept, otherwise reject with the combined reasons.
    ///
    /// Use when combining two criteria that must both be satisfied.
    pub(crate) fn and(self, other: Verdict) -> Verdict {
        use Verdict::*;
        match (self, other) {
            (Accept, Accept) => Accept,
            (Accept, Reject(reasons)) | (Reject(reasons), Accept) => Reject(reasons),
            (Reject(left), Reject(right)) => Reject([left, right].concat()),
        }
    }

    /// Short-hand for creating a single-reason rejection.
    pub(crate) fn reject(reason: Reason) -> Verdict {
        Verdict::Reject(vec![reason])
    }

    /// Fold an iterator of verdicts with [`and`](Self::and): accepts if all accept,
    /// otherwise accumulates all rejection reasons.
    pub(crate) fn all<I: IntoIterator<Item = Verdict>>(iter: I) -> Verdict {
        iter.into_iter().fold(Verdict::Accept, |acc, v| acc.and(v))
    }
}

// Intended as an extension trait for `bool`.
pub(crate) trait VerdictExt {
    fn if_false(self, reason: Reason) -> Verdict;
}

impl VerdictExt for bool {
    /// Reject with `reason` if `self` is false.
    fn if_false(self, reason: Reason) -> Verdict {
        if self {
            Verdict::Accept
        } else {
            Verdict::reject(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a [`ShadowsocksEndpoints`] with no dedicated addresses, backed only by the
    /// WireGuard fallback with the given port ranges.
    fn wg(ipv4: Ipv4Addr) -> WireguardEndpoints {
        WireguardEndpoints {
            ipv4,
            ipv6: None,
            port_ranges: Vec1::try_from_vec(vec![1..=u16::MAX]).unwrap(),
        }
    }

    fn ss_wg_only(port_ranges: Vec<RangeInclusive<u16>>) -> ShadowsocksEndpoints {
        ShadowsocksEndpoints {
            dedicated_addrs: HashSet::new(),
            shadowsocks_port_ranges: Vec1::try_from_vec(port_ranges).ok(),
        }
    }

    /// Build a [`ShadowsocksEndpoints`] with dedicated addresses and a WireGuard fallback.
    fn ss_with_dedicated(
        dedicated: impl IntoIterator<Item = IpAddr>,
        port_ranges: Vec<RangeInclusive<u16>>,
    ) -> ShadowsocksEndpoints {
        ShadowsocksEndpoints {
            dedicated_addrs: dedicated.into_iter().collect(),
            shadowsocks_port_ranges: Vec1::try_from_vec(port_ranges).ok(),
        }
    }

    const PORT_RANGES: &[RangeInclusive<u16>] = &[100..=200, 1000..=2000];
    const WG_IPV4: Ipv4Addr = Ipv4Addr::new(1, 2, 3, 4);

    /// When there are no dedicated Shadowsocks addresses, `random_endpoint` falls back to
    /// the WireGuard address using only the designated Shadowsocks port ranges.
    #[test]
    fn test_shadowsocks_no_dedicated_addrs() {
        let ss = ss_wg_only(PORT_RANGES.to_vec());
        let wg = wg(WG_IPV4);

        // Any port → picks within the WG port ranges.
        let addr = ss
            .random_endpoint(&wg, Constraint::Any, Constraint::Any)
            .expect("should find a valid endpoint without a port constraint");
        assert_eq!(addr.ip(), IpAddr::V4(WG_IPV4));
        assert!(
            PORT_RANGES.iter().any(|r| r.contains(&addr.port())),
            "expected port in range, got {}",
            addr.port()
        );

        // Within-range port → uses that exact port on the WG address.
        let addr = ss
            .random_endpoint(&wg, Constraint::Any, Constraint::Only(100))
            .expect("should find endpoint for within-range port");
        assert_eq!(addr.ip(), IpAddr::V4(WG_IPV4));
        assert_eq!(addr.port(), 100);

        // Out-of-range port → falls through both paths, returns None.
        let result = ss.random_endpoint(&wg, Constraint::Any, Constraint::Only(1));
        assert!(
            result.is_none(),
            "expected None for out-of-range port, got {result:?}"
        );
    }

    /// When dedicated Shadowsocks addresses are present, they are preferred over the
    /// WireGuard fallback, and all ports are valid on them.
    #[test]
    fn test_shadowsocks_dedicated_addrs() {
        let dedicated: Vec<IpAddr> =
            vec!["1.3.3.7".parse().unwrap(), "192.0.2.123".parse().unwrap()];
        let ss = ss_with_dedicated(dedicated.clone(), PORT_RANGES.to_vec());
        let wg = wg(WG_IPV4);

        // Any port → selects a dedicated address (not the WG address).
        let addr = ss
            .random_endpoint(&wg, Constraint::Any, Constraint::Any)
            .expect("should find endpoint without port constraint");
        assert!(
            dedicated.contains(&addr.ip()),
            "expected a dedicated IP, got {}",
            addr.ip()
        );

        // Port outside the WG ranges is still valid on dedicated addresses.
        let addr = ss
            .random_endpoint(&wg, Constraint::Any, Constraint::Only(1))
            .expect("dedicated addrs accept any port");
        assert!(
            dedicated.contains(&addr.ip()),
            "expected a dedicated IP, got {}",
            addr.ip()
        );
        assert_eq!(addr.port(), 1);
    }

    /// Dedicated addresses of the wrong IP family are ignored; selection falls back to
    /// the WireGuard endpoint for the requested IP version.
    #[test]
    fn test_shadowsocks_dedicated_addrs_wrong_ip_family() {
        // Only an IPv6 dedicated address, but we request IPv4.
        let ipv6_dedicated: IpAddr = "::2".parse().unwrap();
        let ss = ss_with_dedicated([ipv6_dedicated], PORT_RANGES.to_vec());
        let wg = wg(WG_IPV4);

        // Any port → falls back to the WG IPv4 address (dedicated addr is wrong family).
        let addr = ss
            .random_endpoint(&wg, Constraint::Only(IpVersion::V4), Constraint::Any)
            .expect("should fall back to WG address");
        assert_eq!(
            addr.ip(),
            IpAddr::V4(WG_IPV4),
            "expected WG address when dedicated addr is wrong family"
        );

        // Out-of-range port → WG fallback cannot satisfy it either → None.
        let result = ss.random_endpoint(&wg, Constraint::Only(IpVersion::V4), Constraint::Only(1));
        assert!(
            result.is_none(),
            "expected None for out-of-range port with wrong-family dedicated addr"
        );

        // Within-range port → WG fallback satisfies it.
        let addr = ss
            .random_endpoint(&wg, Constraint::Only(IpVersion::V4), Constraint::Only(100))
            .expect("WG fallback should handle within-range port");
        assert_eq!(addr.ip(), IpAddr::V4(WG_IPV4));
        assert_eq!(addr.port(), 100);
    }
}
