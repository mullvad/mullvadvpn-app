use std::io::{self, Write};

use ipnetwork::IpNetwork;
use talpid_types::net::{ALLOWED_LAN_MULTICAST_NETS, ALLOWED_LAN_NETS};

pub fn main() {
    generate_allowed_nets_cpp_header(&mut std::io::stdout()).unwrap()
}

/// Generate a C++ header for allowed local network ranges
fn generate_allowed_nets_cpp_header(mut w: impl Write) -> io::Result<()> {
    writeln!(w, "#pragma once")?;
    writeln!(w, "#include <libwfp/ipaddress.h>")?;
    writeln!(w, "#include <libwfp/ipnetwork.h>")?;
    writeln!(w)?;

    let (ipv4_nets, ipv6_nets): (Vec<IpNetwork>, _) =
        ALLOWED_LAN_NETS.iter().partition(|net| net.is_ipv4());
    generate_ip_network_cpp_definitions(&mut w, "g_ipv4LanNets", &ipv4_nets)?;
    generate_ip_network_cpp_definitions(&mut w, "g_ipv6LanNets", &ipv6_nets)?;

    let (ipv4_multicast_nets, ipv6_multicast_nets): (Vec<IpNetwork>, _) =
        ALLOWED_LAN_MULTICAST_NETS
            .iter()
            .partition(|net| net.is_ipv4());
    generate_ip_network_cpp_definitions(&mut w, "g_ipv4MulticastNets", &ipv4_multicast_nets)?;
    generate_ip_network_cpp_definitions(&mut w, "g_ipv6MulticastNets", &ipv6_multicast_nets)?;

    Ok(())
}

fn generate_ip_network_cpp_definitions(
    mut w: impl Write,
    name: &str,
    nets: &[IpNetwork],
) -> io::Result<()> {
    writeln!(w, r"static const wfp::IpNetwork {name}[] = {{")?;

    for net in nets {
        match net {
            IpNetwork::V4(net) => {
                let octets: String = net
                    .ip()
                    .octets()
                    .iter()
                    .map(|oct| oct.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let prefix = net.prefix();
                writeln!(
                    w,
                    "\twfp::IpNetwork(wfp::IpAddress::Literal({{ {octets} }}), {prefix}),"
                )?;
            }
            IpNetwork::V6(net) => {
                write!(w, "\twfp::IpNetwork(wfp::IpAddress::Literal6({{ ")?;
                let mut out = vec![];
                let ip_octets = net.ip().octets();
                let mut iter = ip_octets.iter();
                while let (Some(&a), Some(&b)) = (iter.next(), iter.next()) {
                    out.push(format!("0x{a:02x}{b:02x}"));
                }
                let prefix = net.prefix();
                writeln!(w, "{} }}), {prefix}),", out.join(", "))?;
            }
        }
    }

    writeln!(w, "}};\n")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpp_definition() {
        let mut out = vec![];
        generate_allowed_nets_cpp_header(&mut out).unwrap();
        let out = std::str::from_utf8(&out).unwrap();
        insta::assert_snapshot!(out);
    }
}
