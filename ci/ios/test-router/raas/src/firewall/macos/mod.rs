use std::{
    collections::BTreeMap,
    io,
    net::Ipv4Addr,
    process::{Command, Stdio},
};

use ipnetwork::IpNetwork;
use pfctl::{
    AnchorChange, AnchorKind, FilterRuleAction, FilterRuleBuilder, PfCtl, RedirectRuleAction,
    RedirectRuleBuilder,
};
use tun_rs::{AsyncDevice, DeviceBuilder};

use super::rule::{BlockRule, Endpoints};
use crate::web::routes::TransportProtocol;

const ANCHOR_NAME: &str = "raas";

#[derive(Default)]
pub struct BlockList {
    rules: BTreeMap<uuid::Uuid, Vec<BlockRule>>,
}

impl BlockList {
    pub fn add_rules(&mut self, rules: &[BlockRule], label: uuid::Uuid) -> io::Result<()> {
        let rules_for_label = self.rules.entry(label).or_default();
        rules_for_label.extend_from_slice(rules);
        self.apply_rules()
    }

    pub fn clear_rules_with_label(&mut self, label: &uuid::Uuid) -> io::Result<()> {
        let _ = self.rules.remove(label);
        self.apply_rules()
    }

    pub fn rules(&self) -> &BTreeMap<uuid::Uuid, Vec<BlockRule>> {
        &self.rules
    }

    fn apply_rules(&mut self) -> io::Result<()> {
        let mut pf = PfCtl::new().map_err(pfctl_to_io)?;
        pf.try_enable().map_err(pfctl_to_io)?;
        pf.try_add_anchor(ANCHOR_NAME, AnchorKind::Filter)
            .map_err(pfctl_to_io)?;

        let filter_rules: Vec<pfctl::FilterRule> = self
            .rules
            .values()
            .flatten()
            .flat_map(|rule| create_pf_filter_rules(rule))
            .collect();

        let mut anchor_change = AnchorChange::new();
        anchor_change.set_filter_rules(filter_rules);
        pf.set_rules(ANCHOR_NAME, anchor_change)
            .map_err(pfctl_to_io)?;

        Ok(())
    }
}

fn create_pf_filter_rules(block_rule: &BlockRule) -> Vec<pfctl::FilterRule> {
    match block_rule {
        BlockRule::Host {
            endpoints:
                Endpoints {
                    src,
                    dst,
                    invert_dst,
                },
            protocols,
        } => {
            if protocols.is_empty() {
                create_host_rules(*src, *dst, *invert_dst, None)
            } else {
                protocols
                    .iter()
                    .flat_map(|proto| create_host_rules(*src, *dst, *invert_dst, Some(*proto)))
                    .collect()
            }
        }
        BlockRule::WireGuard {
            endpoints:
                Endpoints {
                    src,
                    dst,
                    invert_dst,
                },
        } => {
            // PF cannot do deep packet inspection for WireGuard headers.
            // Fall back to blocking all UDP from src to dst.
            create_host_rules(*src, *dst, *invert_dst, Some(TransportProtocol::Udp))
        }
    }
}

fn create_host_rules(
    src: IpNetwork,
    dst: IpNetwork,
    invert_dst: bool,
    proto: Option<TransportProtocol>,
) -> Vec<pfctl::FilterRule> {
    let mut rules = Vec::new();

    let af = match src {
        IpNetwork::V4(_) => pfctl::AddrFamily::Ipv4,
        IpNetwork::V6(_) => pfctl::AddrFamily::Ipv6,
    };

    let pf_proto = proto.map(|p| match p {
        TransportProtocol::Tcp => pfctl::Proto::Tcp,
        TransportProtocol::Udp => pfctl::Proto::Udp,
        TransportProtocol::Icmp => pfctl::Proto::Icmp,
        TransportProtocol::IcmpV6 => pfctl::Proto::IcmpV6,
    });

    if invert_dst {
        // Accept traffic matching src+dst, then drop everything else from src
        let mut builder = FilterRuleBuilder::default();
        builder
            .action(FilterRuleAction::Pass)
            .direction(pfctl::Direction::Any)
            .quick(true)
            .af(af)
            .from(pfctl::Ip::from(src))
            .to(pfctl::Ip::from(dst));
        if let Some(proto) = pf_proto {
            builder.proto(proto);
        }
        if let Ok(rule) = builder.build() {
            rules.push(rule);
        }

        let mut builder = FilterRuleBuilder::default();
        builder
            .action(FilterRuleAction::Drop(pfctl::DropAction::Drop))
            .direction(pfctl::Direction::Any)
            .quick(true)
            .af(af)
            .from(pfctl::Ip::from(src));
        if let Some(proto) = pf_proto {
            builder.proto(proto);
        }
        if let Ok(rule) = builder.build() {
            rules.push(rule);
        }
    } else {
        // Drop traffic matching src+dst
        let mut builder = FilterRuleBuilder::default();
        builder
            .action(FilterRuleAction::Drop(pfctl::DropAction::Drop))
            .direction(pfctl::Direction::Any)
            .quick(true)
            .af(af)
            .from(pfctl::Ip::from(src))
            .to(pfctl::Ip::from(dst));
        if let Some(proto) = pf_proto {
            builder.proto(proto);
        }
        if let Ok(rule) = builder.build() {
            rules.push(rule);
        }
    }

    rules
}

/// Hardcoded DNAT rules matching the nftables.nix configuration.
/// All redirect to the same host IP (the gateway/router address).
struct DnatEntry {
    dst_ip: Ipv4Addr,
    dst_port: u16,
    proto: pfctl::Proto,
}

const DNAT_ENTRIES: &[DnatEntry] = &[
    // ip daddr 8.8.8.8 tcp dport 80 dnat to $host
    DnatEntry {
        dst_ip: Ipv4Addr::new(8, 8, 8, 8),
        dst_port: 80,
        proto: pfctl::Proto::Tcp,
    },
    // ip daddr 85.203.53.200 tcp dport 443 dnat to $host
    DnatEntry {
        dst_ip: Ipv4Addr::new(85, 203, 53, 200),
        dst_port: 443,
        proto: pfctl::Proto::Tcp,
    },
];

/// Apply the fixed set of DNAT redirect rules, sending matched traffic to `host_ip`.
pub fn apply_dnat(host_ip: Ipv4Addr) -> io::Result<()> {
    let mut pf = PfCtl::new().map_err(pfctl_to_io)?;
    pf.try_enable().map_err(pfctl_to_io)?;
    pf.try_add_anchor(ANCHOR_NAME, AnchorKind::Redirect)
        .map_err(pfctl_to_io)?;

    let redirect_rules: Result<Vec<pfctl::RedirectRule>, _> = DNAT_ENTRIES
        .iter()
        .map(|entry| {
            RedirectRuleBuilder::default()
                .action(RedirectRuleAction::Redirect)
                .af(pfctl::AddrFamily::Ipv4)
                .proto(entry.proto)
                .to(pfctl::Endpoint::new(
                    entry.dst_ip,
                    pfctl::Port::from(entry.dst_port),
                ))
                .redirect_to(pfctl::Endpoint::new(host_ip, pfctl::Port::Any))
                .build()
        })
        .collect();

    let redirect_rules = redirect_rules.map_err(pfctl_to_io)?;

    let mut anchor_change = AnchorChange::new();
    anchor_change.set_redirect_rules(redirect_rules);
    pf.set_rules(ANCHOR_NAME, anchor_change)
        .map_err(pfctl_to_io)?;

    for entry in DNAT_ENTRIES {
        log::info!(
            "Applied DNAT: {}:{} -> {host_ip}",
            entry.dst_ip,
            entry.dst_port,
        );
    }

    Ok(())
}

/// Remove all DNAT rules from PF.
pub fn cleanup_dnat() {
    let Ok(mut pf) = PfCtl::new() else { return };

    let mut anchor_change = AnchorChange::new();
    anchor_change.set_redirect_rules(vec![]);
    let _ = pf.set_rules(ANCHOR_NAME, anchor_change);
    let _ = pf.remove_anchor(ANCHOR_NAME, AnchorKind::Redirect);
}

fn pfctl_to_io(err: pfctl::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

pub fn setup_utun(client_ip: Ipv4Addr) -> Result<AsyncDevice, io::Error> {
    let device = DeviceBuilder::new()
        .ipv4("10.0.0.2", 24, Some("10.0.0.1"))
        .packet_information(false)
        .build_async()?;
    // .mtu(1500): TODO figure out if we want to dynamically determine MTU to make it equal to
    // the current default route
    let name = device.name()?;
    println!("using utun with name {name}");
    execute_setup_script(client_ip, name)?;
    Ok(device)
}

fn execute_setup_script(client_ip: Ipv4Addr, device_name: String) -> Result<(), io::Error> {
    let status = Command::new("zsh")
        .arg("./poc/post_up.sh")
        .arg(client_ip.to_string())
        .arg(device_name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if let Some(exit_code) = status.code()
        && exit_code != 0
    {
        return Err(io::Error::other(format!(
            "post_up.sh script failed with exit code {exit_code}"
        )));
    }

    Ok(())
}
