use std::{
    collections::BTreeMap,
    io,
    net::Ipv4Addr,
    process::{Command, Stdio},
    sync::Arc,
};

use arc_swap::ArcSwap;
use ipnetwork::IpNetwork;
use pfctl::{
    AnchorChange, AnchorKind, FilterRuleAction, FilterRuleBuilder, PfCtl, RedirectRuleAction,
    RedirectRuleBuilder,
};
use tokio::net::UdpSocket;
use tun_rs::{AsyncDevice, DeviceBuilder};

use super::rule::{BlockRule, Endpoints};
use crate::web::routes::TransportProtocol;

const ANCHOR_NAME: &str = "raas";

/// MTU to route with when the tunnel device will not tell us its own.
const DEFAULT_MTU: u16 = 1500;

mod utun_router;

/// The rules of every label, snapshotted as a whole so a packet is never judged by half an update.
pub(crate) type RuleMap = BTreeMap<uuid::Uuid, Vec<BlockRule>>;

pub struct BlockList {
    rules: Arc<ArcSwap<RuleMap>>,
}

impl BlockList {
    pub(crate) fn new(tunnel_devices: TunnelDevices) -> Self {
        // let mtu = tunnel_device.mtu().unwrap_or_else(|err| {
        //     log::warn!("Failed to read the tunnel device MTU, assuming {DEFAULT_MTU}: {err}");
        //     DEFAULT_MTU
        // });

        let rules = Arc::new(ArcSwap::from_pointee(RuleMap::default()));
        utun_router::Router::spawn(tunnel_devices, rules.clone());
        Self { rules }
    }

    pub fn add_rules(&mut self, rules: &[BlockRule], label: uuid::Uuid) -> io::Result<()> {
        self.rules.rcu(|current| {
            let mut updated = RuleMap::clone(current);
            updated.entry(label).or_default().extend_from_slice(rules);
            updated
        });
        self.apply_rules()
    }

    pub fn clear_rules_with_label(&mut self, label: &uuid::Uuid) -> io::Result<()> {
        self.rules.rcu(|current| {
            let mut updated = RuleMap::clone(current);
            updated.remove(label);
            updated
        });
        self.apply_rules()
    }

    pub fn rules(&self) -> RuleMap {
        RuleMap::clone(&self.rules.load())
    }

    fn apply_rules(&mut self) -> io::Result<()> {
        let mut pf = PfCtl::new().map_err(pfctl_to_io)?;
        pf.try_enable().map_err(pfctl_to_io)?;
        pf.try_add_anchor(ANCHOR_NAME, AnchorKind::Filter)
            .map_err(pfctl_to_io)?;

        let filter_rules: Vec<pfctl::FilterRule> = self
            .rules
            .load()
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

pub struct TunnelDevices {
    pub source: AsyncDevice,
    pub sink: AsyncDevice,
}

pub fn setup_utun(client_ip: Ipv4Addr) -> Result<TunnelDevices, io::Error> {
    // The MTU is left at the device default. It does not have to track the default route's: we
    // terminate the client's TCP connections ourselves, so the client's leg and the upstream leg
    // negotiate their MSS independently and the upstream path MTU never constrains this one.
    let source = DeviceBuilder::new()
        .ipv4("10.0.0.2", 24, Some("10.0.0.1"))
        .packet_information(false)
        .build_async()?;
    let source_name = source.name()?;
    let sink = DeviceBuilder::new()
        .ipv4("10.0.0.3", 24, Some("10.0.0.2"))
        .packet_information(false)
        .build_async()?;
    let sink_name = sink.name()?;
    println!("using source utun with name {source_name}, sink utun with name {sink_name}");
    execute_setup_script(client_ip, source_name, sink_name)?;

    Ok(TunnelDevices { source, sink })
}

fn execute_setup_script(
    client_ip: Ipv4Addr,
    source_name: String,
    sink_name: String,
) -> Result<(), io::Error> {
    let status = Command::new("zsh")
        .arg("./poc/post_up.sh")
        .arg(client_ip.to_string())
        .arg(source_name)
        .arg(sink_name)
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
