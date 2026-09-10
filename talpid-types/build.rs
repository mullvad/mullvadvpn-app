//! Reads `extra-lan-networks.txt` from the repository root and generates the
//! `EXTRA_LAN_NETS` constant that `src/net/allowed_nets.rs` appends to the
//! app's list of local networks.
//!
//! The file format is documented in `extra-lan-networks.txt` itself. In short:
//! one network per line, `#` starts a comment, a bare address means a single
//! host (`/32` or `/128`), and a CIDR range must start on its network address.
//!
//! Only non-public address space is accepted. This is a deliberate guard: the
//! list exempts traffic from the tunnel, so a typo must never be able to
//! exempt real internet addresses.

use std::{
    env, fs,
    io::Write,
    net::{Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
};

const FILE_NAME: &str = "extra-lan-networks.txt";

/// Address space that entries are allowed to fall within.
/// (address, prefix) pairs; the entry must be fully contained in one of them.
const PERMITTED_V4: &[(Ipv4Addr, u8)] = &[
    (Ipv4Addr::new(10, 0, 0, 0), 8),     // Private (RFC 1918)
    (Ipv4Addr::new(172, 16, 0, 0), 12),  // Private (RFC 1918)
    (Ipv4Addr::new(192, 168, 0, 0), 16), // Private (RFC 1918)
    (Ipv4Addr::new(169, 254, 0, 0), 16), // Link-local
    (Ipv4Addr::new(100, 64, 0, 0), 10), // Shared address space / CGNAT (RFC 6598), used by Tailscale
];
const PERMITTED_V6: &[(Ipv6Addr, u8)] = &[
    (Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10), // Link-local
    (Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),  // Unique local (ULA)
];

#[derive(Debug, Clone, Copy)]
enum Net {
    V4(Ipv4Addr, u8),
    V6(Ipv6Addr, u8),
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = manifest_dir.join("..").join(FILE_NAME);
    println!("cargo:rerun-if-changed={}", path.display());

    let nets = match fs::read_to_string(&path) {
        Ok(contents) => parse(&contents).unwrap_or_else(|err| {
            panic!("{}: {err}", path.display());
        }),
        Err(err) => {
            println!(
                "cargo:warning={}: {err}; building without extra LAN networks",
                path.display()
            );
            vec![]
        }
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    write_rust(&out_dir.join("extra_lan_nets.rs"), &nets).unwrap();
}

fn parse(contents: &str) -> Result<Vec<Net>, String> {
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

fn parse_line(line: &str) -> Result<Net, String> {
    let (addr, prefix) = match line.split_once('/') {
        Some((addr, prefix)) => (addr.trim(), Some(prefix.trim())),
        None => (line, None),
    };

    let net = if let Ok(v4) = addr.parse::<Ipv4Addr>() {
        let prefix = parse_prefix(prefix, 32)?;
        let host_bits = u32::from(v4) & !mask32(prefix);
        if host_bits != 0 {
            return Err(format!(
                "\"{line}\" has host bits set. For a single device write \"{addr}\" \
                 (or \"{addr}/32\"); for a range start it at its network address, e.g. \"{}/{prefix}\"",
                Ipv4Addr::from(u32::from(v4) & mask32(prefix))
            ));
        }
        Net::V4(v4, prefix)
    } else if let Ok(v6) = addr.parse::<Ipv6Addr>() {
        let prefix = parse_prefix(prefix, 128)?;
        let host_bits = u128::from(v6) & !mask128(prefix);
        if host_bits != 0 {
            return Err(format!(
                "\"{line}\" has host bits set. For a single device write \"{addr}\" \
                 (or \"{addr}/128\"); for a range start it at its network address, e.g. \"{}/{prefix}\"",
                Ipv6Addr::from(u128::from(v6) & mask128(prefix))
            ));
        }
        Net::V6(v6, prefix)
    } else {
        return Err(format!(
            "\"{line}\" is not an IPv4 or IPv6 address or network (expected e.g. \
             \"100.101.102.103\" or \"100.64.0.0/10\")"
        ));
    };

    if !is_permitted(net) {
        return Err(format!(
            "\"{line}\" is outside the private, link-local, unique-local and shared (CGNAT) \
             ranges. Public internet addresses cannot be treated as local. \
             See PERMITTED_V4/PERMITTED_V6 in talpid-types/build.rs"
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

fn mask32(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - u32::from(prefix))
    }
}

fn mask128(prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - u32::from(prefix))
    }
}

/// An entry is permitted if it lies entirely inside one of the permitted blocks,
/// i.e. it is at least as specific as the block and shares the block's network bits.
fn is_permitted(net: Net) -> bool {
    match net {
        Net::V4(addr, prefix) => PERMITTED_V4.iter().any(|&(block, block_prefix)| {
            prefix >= block_prefix && (u32::from(addr) & mask32(block_prefix)) == u32::from(block)
        }),
        Net::V6(addr, prefix) => PERMITTED_V6.iter().any(|&(block, block_prefix)| {
            prefix >= block_prefix
                && (u128::from(addr) & mask128(block_prefix)) == u128::from(block)
        }),
    }
}

fn write_rust(path: &Path, nets: &[Net]) -> std::io::Result<()> {
    let mut out = fs::File::create(path)?;
    writeln!(
        out,
        "// Generated by talpid-types/build.rs from {FILE_NAME}. Do not edit."
    )?;
    writeln!(
        out,
        "/// Networks listed in `{FILE_NAME}`, appended to [`BASE_LAN_NETS`] to form [`ALLOWED_LAN_NETS`]."
    )?;
    writeln!(
        out,
        "pub const EXTRA_LAN_NETS: [IpNetwork; {}] = [",
        nets.len()
    )?;
    for net in nets {
        match *net {
            Net::V4(addr, prefix) => {
                let [a, b, c, d] = addr.octets();
                writeln!(out, "    v4(Ipv4Addr::new({a}, {b}, {c}, {d}), {prefix}),")?;
            }
            Net::V6(addr, prefix) => {
                let s = addr.segments();
                writeln!(
                    out,
                    "    v6(Ipv6Addr::new({:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}, {:#06x}), {prefix}),",
                    s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]
                )?;
            }
        }
    }
    writeln!(out, "];")?;
    Ok(())
}
