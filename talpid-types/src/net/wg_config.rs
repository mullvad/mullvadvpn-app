//! Parser for the `wg-quick` config file format (sans DNS and some other things).
//!
//! The parser is exposed as `impl FromStr for CustomVpnConfig`, so a `.conf` file can be
//! parsed with `CustomVpnConfig::from_str(...)`. The peer `Endpoint` must be an IP address
//! and port; hostnames are not resolved.

use std::str::FromStr;

use ipnetwork::IpNetwork;

use crate::net::wireguard::{
    CustomVpnConfig, CustomVpnPeerConfig, CustomVpnTunnelConfig, InvalidKey, PrivateKey, PublicKey,
};

/// Errors produced when parsing a wg-quick config into a [`CustomVpnConfig`].
#[derive(Debug, thiserror::Error)]
pub enum WgConfigParseError {
    #[error("Unknown section: '{0}' (expected [Interface] or [Peer])")]
    UnknownSection(String),
    // NOTE: These errors must not include values, since they may contain the private key.
    #[error("Key '{key}' outside of any section on line {line}")]
    OrphanKeyValue { key: String, line: usize },
    #[error("Missing '=' on line {0}")]
    MissingEquals(usize),
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
    #[error("Invalid Endpoint '{0}' (expected <host>:<port>)")]
    InvalidEndpoint(String),
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
    endpoint: Option<String>,
}

enum Section {
    None,
    Interface,
    Peer,
}

/// Recognized fields:
/// * `[Interface]`: `PrivateKey`, `Address` - the first address is taken as the
///   tunnel IP; any CIDR suffix is stripped.
/// * `[Peer]`: `PublicKey`, `AllowedIPs`, `Endpoint` - `Endpoint` is stored as a
///   `<host>:<port>` string, and host must be an IP literal.
///
/// Other wg-quick fields (`ListenPort`, `DNS`, `MTU`, `PresharedKey`,
/// `PersistentKeepalive`, `PreUp`/`PostUp`/`PreDown`/`PostDown`, etc.) are
/// accepted but ignored. Exactly one `[Peer]` section is required -
/// the config only holds a single peer.
///
/// Keys and section names are matched case-insensitively. Lines starting with
/// `#` and trailing `#`-comments are stripped. Blank lines are ignored.
impl FromStr for CustomVpnConfig {
    type Err = WgConfigParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use WgConfigParseError as E;

        let mut section = Section::None;
        let mut interface = InterfaceBuilder::default();
        let mut peer: Option<PeerBuilder> = None;

        for (line_index, raw_line) in s.lines().enumerate() {
            let line_number = line_index + 1;
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
                .ok_or(E::MissingEquals(line_number))?;
            let key = key.trim();
            let value = value.trim();

            match section {
                Section::None => {
                    return Err(E::OrphanKeyValue {
                        key: key.to_owned(),
                        line: line_number,
                    });
                }
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
        let trimmed = value.trim();
        if trimmed.is_empty() || !trimmed.contains(':') {
            return Err(E::InvalidEndpoint(trimmed.to_owned()));
        }
        peer.endpoint = Some(trimmed.to_owned());
    }
    Ok(())
}

fn finalize(
    interface: InterfaceBuilder,
    peer: Option<PeerBuilder>,
) -> Result<CustomVpnConfig, WgConfigParseError> {
    use WgConfigParseError as E;

    let private_key = interface.private_key.ok_or(E::MissingField("PrivateKey"))?;
    if interface.addresses.is_empty() {
        return Err(E::MissingField("Address"));
    }
    let tunnel_ips: Vec<_> = interface.addresses.iter().map(|net| net.ip()).collect();

    let peer = peer.ok_or(E::MissingPeer)?;
    let public_key = peer.public_key.ok_or(E::MissingField("PublicKey"))?;
    if peer.allowed_ips.is_empty() {
        return Err(E::MissingField("AllowedIPs"));
    }
    let endpoint = peer.endpoint.ok_or(E::MissingField("Endpoint"))?;

    Ok(CustomVpnConfig {
        tunnel: CustomVpnTunnelConfig {
            private_key,
            ips: tunnel_ips,
        },
        peer: CustomVpnPeerConfig {
            public_key,
            allowed_ip: peer.allowed_ips,
            endpoint: endpoint
                .parse()
                .map_err(|_| WgConfigParseError::InvalidEndpoint(endpoint))?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid 32-byte base64 strings. Content is arbitrary - x25519 keys are
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
        let config = CustomVpnConfig::from_str(&sample_config()).unwrap();
        assert_eq!(config.tunnel.ips.len(), 1);
        assert_eq!(config.tunnel.ips[0].to_string(), "10.0.0.2");
        assert_eq!(config.peer.allowed_ip.len(), 2);
        assert_eq!(config.peer.endpoint, "1.2.3.4:51820".parse().unwrap());
    }

    #[test]
    fn address_cidr_is_stripped_for_tunnel_ip() {
        let config = CustomVpnConfig::from_str(&sample_config()).unwrap();
        assert_eq!(config.tunnel.ips[0].to_string(), "10.0.0.2");
    }

    #[test]
    fn dual_stack_addresses_are_preserved() {
        let input = format!(
            "\
[Interface]
PrivateKey = {PRIV}
Address = 10.0.0.2/24, fd00::2/64

[Peer]
PublicKey = {PUB}
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = 1.2.3.4:51820
"
        );
        let config = CustomVpnConfig::from_str(&input).unwrap();
        let ips: Vec<String> = config.tunnel.ips.iter().map(ToString::to_string).collect();
        assert_eq!(ips, vec!["10.0.0.2", "fd00::2"]);
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
        CustomVpnConfig::from_str(&input).unwrap();
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
        CustomVpnConfig::from_str(&input).unwrap();
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
        CustomVpnConfig::from_str(&input).unwrap();
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
            CustomVpnConfig::from_str(&input).unwrap_err(),
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
            CustomVpnConfig::from_str(&input).unwrap_err(),
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
            CustomVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::MultiplePeers,
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
            CustomVpnConfig::from_str(&input).unwrap_err(),
            WgConfigParseError::UnknownSection(ref s) if s == "Bogus",
        ));
    }
}
