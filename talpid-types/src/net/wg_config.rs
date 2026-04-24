//! Parser for the `wg-quick` config file format (sans DNS and some other things).
//!
//! The parser is exposed as `impl FromStr for PersonalVpnConfig` so any crate
//! in the workspace can call `PersonalVpnConfig::from_str(...)` on the contents
//! of a `.conf` file.

use std::{net::SocketAddr, str::FromStr};

use ipnetwork::IpNetwork;

use crate::net::wireguard::{
    InvalidKey, PersonalVpnConfig, PersonalVpnPeerConfig, PersonalVpnTunnelConfig, PrivateKey,
    PublicKey,
};

/// Errors produced when parsing a wg-quick config into a [`PersonalVpnConfig`].
#[derive(Debug, thiserror::Error)]
pub enum WgConfigParseError {
    #[error("Unknown section: '{0}' (expected [Interface] or [Peer])")]
    UnknownSection(String),
    #[error("Key-value pair outside of any section: '{0}'")]
    OrphanKeyValue(String),
    #[error("Missing '=' in line: '{0}'")]
    MissingEquals(String),
    #[error("Missing required field '{0}'")]
    MissingField(&'static str),
    #[error("Duplicate field '{0}'")]
    DuplicateField(&'static str),
    #[error("Missing [Peer] section")]
    MissingPeer,
    #[error("Expected exactly one [Peer] section")]
    MultiplePeers,
    #[error("Invalid PrivateKey: {0}")]
    InvalidPrivateKey(#[source] InvalidKey),
    #[error("Invalid PublicKey: {0}")]
    InvalidPublicKey(#[source] InvalidKey),
    #[error("Invalid Address '{value}': {source}")]
    InvalidAddress {
        value: String,
        #[source]
        source: ipnetwork::IpNetworkError,
    },
    #[error("Invalid AllowedIPs entry '{value}': {source}")]
    InvalidAllowedIp {
        value: String,
        #[source]
        source: ipnetwork::IpNetworkError,
    },
    #[error("Invalid Endpoint '{value}' (hostnames are not supported; use <ip>:<port>): {source}")]
    InvalidEndpoint {
        value: String,
        #[source]
        source: std::net::AddrParseError,
    },
}

#[derive(Default)]
struct InterfaceBuilder {
    private_key: Option<PrivateKey>,
    addresses: Vec<IpNetwork>,
}

#[derive(Default)]
struct PeerBuilder {
    public_key: Option<PublicKey>,
    allowed_ips: Vec<IpNetwork>,
    endpoint: Option<SocketAddr>,
}

enum Section {
    None,
    Interface,
    Peer,
}

/// Recognized fields:
/// * `[Interface]`: `PrivateKey`, `Address` — the first address is taken as the
///   tunnel IP; any CIDR suffix is stripped.
/// * `[Peer]`: `PublicKey`, `AllowedIPs`, `Endpoint` - `Endpoint` must be a
///   numeric `<ip>:<port>` (hostnames would require DNS resolution and are
///   rejected here).
///
/// Other wg-quick fields (`ListenPort`, `DNS`, `MTU`, `PresharedKey`,
/// `PersistentKeepalive`, `PreUp`/`PostUp`/`PreDown`/`PostDown`, etc.) are
/// accepted but ignored. Exactly one `[Peer]` section is required -
/// `PersonalVpnConfig` only holds a single peer.
///
/// Keys and section names are matched case-insensitively. Lines starting with
/// `#` and trailing `#`-comments are stripped. Blank lines are ignored.
impl FromStr for PersonalVpnConfig {
    type Err = WgConfigParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WgConfigParseError as E;

        let mut section = Section::None;
        let mut interface = InterfaceBuilder::default();
        let mut peer: Option<PeerBuilder> = None;

        for raw_line in s.lines() {
            let line = raw_line
                .split_once('#')
                .map_or(raw_line, |(before, _)| before);
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = parse_section_header(header.trim())?;
                if matches!(section, Section::Peer) {
                    if peer.is_some() {
                        return Err(E::MultiplePeers);
                    }
                    peer = Some(PeerBuilder::default());
                }
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| E::MissingEquals(line.to_owned()))?;
            let key = key.trim();
            let value = value.trim();

            match section {
                Section::None => return Err(E::OrphanKeyValue(line.to_owned())),
                Section::Interface => handle_interface_line(&mut interface, key, value)?,
                Section::Peer => {
                    let peer = peer.as_mut().expect("Section::Peer implies a peer");
                    handle_peer_line(peer, key, value)?;
                }
            }
        }

        finalize(interface, peer)
    }
}

fn parse_section_header(header: &str) -> Result<Section, WgConfigParseError> {
    if header.eq_ignore_ascii_case("Interface") {
        Ok(Section::Interface)
    } else if header.eq_ignore_ascii_case("Peer") {
        Ok(Section::Peer)
    } else {
        Err(WgConfigParseError::UnknownSection(header.to_owned()))
    }
}

fn handle_interface_line(
    interface: &mut InterfaceBuilder,
    key: &str,
    value: &str,
) -> Result<(), WgConfigParseError> {
    use WgConfigParseError as E;

    if key.eq_ignore_ascii_case("PrivateKey") {
        if interface.private_key.is_some() {
            return Err(E::DuplicateField("PrivateKey"));
        }
        interface.private_key = Some(PrivateKey::from_base64(value).map_err(E::InvalidPrivateKey)?);
    } else if key.eq_ignore_ascii_case("Address") {
        for addr in value.split(',') {
            let addr = addr.trim();
            if addr.is_empty() {
                continue;
            }
            let net = addr
                .parse::<IpNetwork>()
                .map_err(|source| E::InvalidAddress {
                    value: addr.to_owned(),
                    source,
                })?;
            interface.addresses.push(net);
        }
    }
    Ok(())
}

fn handle_peer_line(
    peer: &mut PeerBuilder,
    key: &str,
    value: &str,
) -> Result<(), WgConfigParseError> {
    use WgConfigParseError as E;

    if key.eq_ignore_ascii_case("PublicKey") {
        if peer.public_key.is_some() {
            return Err(E::DuplicateField("PublicKey"));
        }
        peer.public_key = Some(PublicKey::from_base64(value).map_err(E::InvalidPublicKey)?);
    } else if key.eq_ignore_ascii_case("AllowedIPs") {
        for entry in value.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let net = entry
                .parse::<IpNetwork>()
                .map_err(|source| E::InvalidAllowedIp {
                    value: entry.to_owned(),
                    source,
                })?;
            peer.allowed_ips.push(net);
        }
    } else if key.eq_ignore_ascii_case("Endpoint") {
        if peer.endpoint.is_some() {
            return Err(E::DuplicateField("Endpoint"));
        }
        peer.endpoint = Some(
            value
                .parse::<SocketAddr>()
                .map_err(|source| E::InvalidEndpoint {
                    value: value.to_owned(),
                    source,
                })?,
        );
    }
    Ok(())
}

fn finalize(
    interface: InterfaceBuilder,
    peer: Option<PeerBuilder>,
) -> Result<PersonalVpnConfig, WgConfigParseError> {
    use WgConfigParseError as E;

    let private_key = interface.private_key.ok_or(E::MissingField("PrivateKey"))?;
    // TODO: include all addresses
    let tunnel_ip = interface
        .addresses
        .first()
        .ok_or(E::MissingField("Address"))?
        .ip();

    let peer = peer.ok_or(E::MissingPeer)?;
    let public_key = peer.public_key.ok_or(E::MissingField("PublicKey"))?;
    if peer.allowed_ips.is_empty() {
        return Err(E::MissingField("AllowedIPs"));
    }
    let endpoint = peer.endpoint.ok_or(E::MissingField("Endpoint"))?;

    Ok(PersonalVpnConfig {
        tunnel: PersonalVpnTunnelConfig {
            private_key,
            ip: tunnel_ip,
        },
        peer: PersonalVpnPeerConfig {
            public_key,
            allowed_ip: peer.allowed_ips,
            endpoint,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid 32-byte base64 strings. Content is arbitrary — x25519 keys are
    // not validated for strength at parse time.
    const PRIV: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const PUB: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBE=";

    fn sample_config() -> String {
        format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2/24

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = 1.2.3.4:51820
"
        )
    }

    #[test]
    fn happy_path() {
        let config = PersonalVpnConfig::from_str(&sample_config()).unwrap();
        assert_eq!(config.tunnel.ip.to_string(), "10.0.0.2");
        assert_eq!(config.peer.allowed_ip.len(), 2);
        assert_eq!(config.peer.endpoint.to_string(), "1.2.3.4:51820");
    }

    #[test]
    fn address_cidr_is_stripped_for_tunnel_ip() {
        let config = PersonalVpnConfig::from_str(&sample_config()).unwrap();
        assert_eq!(config.tunnel.ip.to_string(), "10.0.0.2");
    }

    #[test]
    fn keys_are_case_insensitive() {
        let input = format!(
            "\
[interface]
privatekey = {PRIV}
address = 10.0.0.2

[PEER]
PUBLICKEY = {PUB}
allowedips = 0.0.0.0/0
ENDPOINT = 1.2.3.4:51820
"
        );
        PersonalVpnConfig::from_str(&input).unwrap();
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let input = format!(
            "\
# top-level comment
[Interface]
PrivateKey = {PRIV} # inline comment
Address = 10.0.0.2

# between sections

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820
"
        );
        PersonalVpnConfig::from_str(&input).unwrap();
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2
ListenPort = 51820
DNS = 1.1.1.1
MTU = 1420

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820
PersistentKeepalive = 25
"
        );
        PersonalVpnConfig::from_str(&input).unwrap();
    }

    #[test]
    fn missing_address_is_rejected() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820
"
        );
        assert!(matches!(
            PersonalVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::MissingField("Address"),
        ));
    }

    #[test]
    fn duplicate_private_key_is_rejected() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
PrivateKey = {PRIV}
Address = 10.0.0.2

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820
"
        );
        assert!(matches!(
            PersonalVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::DuplicateField("PrivateKey"),
        ));
    }

    #[test]
    fn multiple_peers_are_rejected() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 5.6.7.8:51820
"
        );
        assert!(matches!(
            PersonalVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::MultiplePeers,
        ));
    }

    #[test]
    fn hostname_endpoint_is_rejected() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = vpn.example.com:51820
"
        );
        assert!(matches!(
            PersonalVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::InvalidEndpoint { .. },
        ));
    }

    #[test]
    fn unknown_section_is_rejected() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2

[Bogus]
Foo = bar

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0
Endpoint = 1.2.3.4:51820
"
        );
        assert!(matches!(
            PersonalVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::UnknownSection(ref s) if s == "Bogus",
        ));
    }
}
