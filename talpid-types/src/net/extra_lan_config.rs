//! Parser for `extra-lan-networks.txt`, the file in the repository root that lists
//! additional networks this build treats as local. See that file for the format.
//!
//! Used by the `generate-extra-lan-nets` binary to produce `extra_lan_nets.rs`, and by
//! the tests to check that the generated file matches the text file.
//!
//! Only non-public address space is accepted. This is a deliberate guard: the list
//! exempts traffic from the tunnel, so a typo must never be able to exempt real
//! internet addresses.

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// File name of the configuration file, relative to the repository root.
pub const EXTRA_LAN_NETWORKS_FILE: &str = "extra-lan-networks.txt";

/// Address space that entries are allowed to fall within. An entry must be fully
/// contained in one of these.
pub const PERMITTED_NETS: [IpNetwork; 7] = [
    // Private (RFC 1918)
    v4(Ipv4Addr::new(10, 0, 0, 0), 8),
    v4(Ipv4Addr::new(172, 16, 0, 0), 12),
    v4(Ipv4Addr::new(192, 168, 0, 0), 16),
    // Link-local
    v4(Ipv4Addr::new(169, 254, 0, 0), 16),
    // Shared address space / CGNAT (RFC 6598), used by Tailscale
    v4(Ipv4Addr::new(100, 64, 0, 0), 10),
    // Link-local IPv6
    v6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10),
    // Unique local (ULA)
    v6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),
];

const fn v4(address: Ipv4Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V4(Ipv4Network::new_checked(address, prefix).unwrap())
}

const fn v6(address: Ipv6Addr, prefix: u8) -> IpNetwork {
    IpNetwork::V6(Ipv6Network::new_checked(address, prefix).unwrap())
}

/// Parse the contents of `extra-lan-networks.txt`.
///
/// One entry per line, `#` starts a comment, blank lines are ignored. A bare address
/// means a single host (`/32` or `/128`). A CIDR range must start at its network
/// address. Every entry must lie inside [`PERMITTED_NETS`].
pub fn parse(contents: &str) -> Result<Vec<IpNetwork>, String> {
    let mut nets = vec![];
    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let net = parse_line(line).map_err(|err| format!("line {}: {err}", index + 1))?;
        nets.push(net);
    }
    Ok(nets)
}

fn parse_line(line: &str) -> Result<IpNetwork, String> {
    let (addr, prefix) = match line.split_once('/') {
        Some((addr, prefix)) => (addr.trim(), Some(prefix.trim())),
        None => (line, None),
    };

    let addr: IpAddr = addr.parse().map_err(|_| {
        format!(
            "\"{line}\" is not an IPv4 or IPv6 address or network (expected e.g. \
             \"100.101.102.103\" or \"100.64.0.0/10\")"
        )
    })?;
    let max_prefix = if addr.is_ipv4() { 32 } else { 128 };
    let prefix = parse_prefix(prefix, max_prefix)?;

    let net = IpNetwork::new(addr, prefix).map_err(|err| err.to_string())?;
    if net.network() != addr {
        let single = if addr.is_ipv4() { "/32" } else { "/128" };
        return Err(format!(
            "\"{line}\" has host bits set. For a single device write \"{addr}\" \
             (or \"{addr}{single}\"); for a range start it at its network address, \
             e.g. \"{}/{prefix}\"",
            net.network()
        ));
    }

    if !is_permitted(net) {
        return Err(format!(
            "\"{line}\" is outside the private, link-local, unique-local and shared (CGNAT) \
             ranges. Public internet addresses cannot be treated as local. \
             See PERMITTED_NETS in talpid-types/src/net/extra_lan_config.rs"
        ));
    }
    Ok(net)
}

fn parse_prefix(prefix: Option<&str>, max: u8) -> Result<u8, String> {
    match prefix {
        None => Ok(max),
        Some(p) => {
            let prefix: u8 = p
                .parse()
                .map_err(|_| format!("\"/{p}\" is not a valid prefix length (0-{max})"))?;
            if prefix > max {
                return Err(format!("prefix length /{prefix} is larger than /{max}"));
            }
            Ok(prefix)
        }
    }
}

/// An entry is permitted if it lies entirely inside one of the permitted blocks.
pub fn is_permitted(net: IpNetwork) -> bool {
    PERMITTED_NETS.iter().any(|block| match (block, net) {
        (IpNetwork::V4(block), IpNetwork::V4(net)) => {
            net.prefix() >= block.prefix() && block.contains(net.ip())
        }
        (IpNetwork::V6(block), IpNetwork::V6(net)) => {
            net.prefix() >= block.prefix() && block.contains(net.ip())
        }
        _ => false,
    })
}

/// Render the parsed networks as the Rust source of `extra_lan_nets.rs`.
pub fn render_rust(nets: &[IpNetwork]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// Generated from {EXTRA_LAN_NETWORKS_FILE} by \
         `cargo run -p talpid-types --bin generate-extra-lan-nets`. Do not edit.\n"
    ));
    out.push_str(&format!(
        "/// Networks listed in `{EXTRA_LAN_NETWORKS_FILE}`, appended to [`BASE_LAN_NETS`] \
         to form [`ALLOWED_LAN_NETS`].\n"
    ));
    out.push_str(&format!(
        "pub const EXTRA_LAN_NETS: [IpNetwork; {}] = [\n",
        nets.len()
    ));
    for net in nets {
        match net {
            IpNetwork::V4(net) => {
                let [a, b, c, d] = net.ip().octets();
                out.push_str(&format!(
                    "    v4(Ipv4Addr::new({a}, {b}, {c}, {d}), {}),\n",
                    net.prefix()
                ));
            }
            IpNetwork::V6(net) => {
                let s = net.ip().segments();
                out.push_str(&format!(
                    "    v6(Ipv6Addr::new({:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}), {}),\n",
                    s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], net.prefix()
                ));
            }
        }
    }
    out.push_str("];\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hosts_and_ranges() {
        let nets = parse(
            "# comment\n100.101.102.103\n100.101.102.104/32 # trailing\n\n  100.81.0.0/24  \n\
             fd7a:115c:a1e0::1234:5678\n192.168.1.0/24\n",
        )
        .unwrap();
        assert_eq!(nets.len(), 5);
        assert_eq!(nets[0].to_string(), "100.101.102.103/32");
        assert_eq!(nets[3].to_string(), "fd7a:115c:a1e0::1234:5678/128");
    }

    #[test]
    fn rejects_bad_input() {
        for bad in [
            "100.64.1.5/10",           // host bits set
            "8.8.8.8",                 // public
            "2001:db8::1",             // public
            "100.64.0.0/33",           // bad prefix
            "banana",                  // not an address
            "100.63.255.255",          // just outside CGNAT
            "fd7a:115c:a1e0::1234/64", // host bits set
        ] {
            assert!(parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn empty_file_gives_empty_list() {
        assert!(parse("# nothing here\n\n").unwrap().is_empty());
    }
}
