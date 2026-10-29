//! The firewall rules.
//!
//! Each rule installs one or more WFP filters inside an ongoing transaction.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};

use talpid_types::net::{ALLOWED_LAN_MULTICAST_NETS, ALLOWED_LAN_NETS};
use wfp::{
    ActionType, AppIdConditionBuilder, FilterBuilder, FilterWeight, IcmpConditionBuilder,
    InterfaceConditionBuilder, IpAddressConditionBuilder, Layer, LoopbackConditionBuilder,
    PortConditionBuilder, ProtocolConditionBuilder, Transaction, WeightRange,
};

use crate::{
    guids::*,
    session::{Record, Recorder},
};

/// Well-known ports.
mod ports {
    pub const DHCP_V4_CLIENT_PORT: u16 = 68;
    pub const DHCP_V4_SERVER_PORT: u16 = 67;
    pub const DHCP_V6_CLIENT_PORT: u16 = 546;
    pub const DHCP_V6_SERVER_PORT: u16 = 547;
    pub const DNS_SERVER_PORT: u16 = 53;
}

/// The transport protocols the firewall can restrict traffic by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    fn condition(self) -> wfp::Condition {
        match self {
            Self::Tcp => ProtocolConditionBuilder::tcp().build(),
            Self::Udp => ProtocolConditionBuilder::udp().build(),
        }
    }
}

/// A network endpoint the firewall permits traffic to or from.
#[derive(Clone, Debug)]
pub struct RelayEndpoint {
    pub ip: IpAddr,
    pub port: u16,
    pub protocol: Protocol,
}

/// The sublayer a rule belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sublayer {
    Baseline,
    Dns,
}

impl Sublayer {
    fn guid(self) -> wfp::Guid {
        match self {
            Self::Baseline => SUBLAYER_BASELINE,
            Self::Dns => SUBLAYER_DNS,
        }
    }
}

/// The traffic that is allowed inside the tunnel while connecting.
#[derive(Clone, Debug, Default)]
pub enum AllowedTunnelTraffic {
    /// Allow no traffic in the tunnel.
    #[default]
    None,
    /// Allow all traffic in the tunnel.
    All,
    /// Allow traffic to one endpoint in the tunnel.
    One(RelayEndpoint),
    /// Allow traffic to two endpoints in the tunnel.
    Two(RelayEndpoint, RelayEndpoint),
}

/// One firewall rule: a parameter bundle plus the filters it installs.
#[derive(Clone, Debug)]
pub enum Rule {
    /// Block all traffic in the baseline sublayer.
    BlockAll,
    /// Permit loopback traffic.
    PermitLoopback,
    /// Permit DHCP client traffic.
    PermitDhcp,
    /// Permit NDP router and neighbor discovery.
    PermitNdp,
    /// Permit serving DHCP requests from the LAN.
    PermitDhcpServer,
    /// Permit outbound traffic to the LAN.
    PermitLan,
    /// Permit inbound traffic from the LAN.
    PermitLanService,
    /// Lift all DNS out of the baseline sublayer, to be restricted in the DNS
    /// sublayer instead.
    PermitDns,
    /// Permit outbound traffic inside the tunnel.
    PermitVpnTunnel {
        clients: Vec<PathBuf>,
        alias: String,
        endpoints: Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    },
    /// Permit inbound traffic inside the tunnel.
    PermitVpnTunnelService {
        clients: Vec<PathBuf>,
        alias: String,
        endpoints: Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    },
    /// Block all DNS in the DNS sublayer.
    DnsBlockAll,
    /// Permit DNS on loopback.
    DnsPermitLoopback,
    /// Permit DNS to the given servers inside the tunnel.
    DnsPermitTunnel { alias: String, hosts: Vec<IpAddr> },
    /// Permit DNS to the given servers outside the tunnel.
    DnsPermitNonTunnel {
        alias: Option<String>,
        hosts: Vec<IpAddr>,
    },
    /// Permit outbound traffic to an endpoint, in the given sublayer.
    PermitEndpoint {
        endpoint: RelayEndpoint,
        clients: Vec<PathBuf>,
        sublayer: Sublayer,
    },
}

/// Everything an installed rule needs: the transaction to add objects in, and
/// the record of objects added so far.
pub struct Cx<'a> {
    pub transaction: &'a Transaction<'a>,
    pub recorder: &'a mut Recorder,
}

impl Rule {
    /// Install the rule's filters, recording them for reversion.
    pub fn install(&self, cx: &mut Cx<'_>) -> io::Result<()> {
        match self {
            Self::BlockAll => Self::install_block_all(cx),
            Self::PermitLoopback => Self::install_permit_loopback(cx),
            Self::PermitDhcp => Self::install_permit_dhcp(cx),
            Self::PermitNdp => Self::install_permit_ndp(cx),
            Self::PermitDhcpServer => Self::install_permit_dhcp_server(cx),
            Self::PermitLan => Self::install_permit_lan(cx),
            Self::PermitLanService => Self::install_permit_lan_service(cx),
            Self::PermitDns => Self::install_permit_dns(cx),
            Self::PermitVpnTunnel {
                clients,
                alias,
                endpoints,
                exit_ip,
            } => Self::install_permit_vpn_tunnel(cx, clients, alias, endpoints, *exit_ip),
            Self::PermitVpnTunnelService {
                clients,
                alias,
                endpoints,
                exit_ip,
            } => Self::install_permit_vpn_tunnel_service(cx, clients, alias, endpoints, *exit_ip),
            Self::DnsBlockAll => Self::install_dns_block_all(cx),
            Self::DnsPermitLoopback => Self::install_dns_permit_loopback(cx),
            Self::DnsPermitTunnel { alias, hosts } => {
                Self::install_dns_permit_tunnel(cx, alias, hosts)
            }
            Self::DnsPermitNonTunnel { alias, hosts } => {
                Self::install_dns_permit_non_tunnel(cx, alias, hosts)
            }
            Self::PermitEndpoint {
                endpoint,
                clients,
                sublayer,
            } => Self::install_permit_endpoint(cx, endpoint, clients, *sublayer),
        }
    }

    fn install_block_all(cx: &mut Cx<'_>) -> io::Result<()> {
        for (guid, layer, name) in [
            (
                FILTER_BASELINE_BLOCK_ALL_OUTBOUND_IPV4,
                Layer::ConnectV4,
                "Block all outbound connections (IPv4)",
            ),
            (
                FILTER_BASELINE_BLOCK_ALL_INBOUND_IPV4,
                Layer::AcceptV4,
                "Block all inbound connections (IPv4)",
            ),
            (
                FILTER_BASELINE_BLOCK_ALL_OUTBOUND_IPV6,
                Layer::ConnectV6,
                "Block all outbound connections (IPv6)",
            ),
            (
                FILTER_BASELINE_BLOCK_ALL_INBOUND_IPV6,
                Layer::AcceptV6,
                "Block all inbound connections (IPv6)",
            ),
        ] {
            let id = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description(
                    "This filter is part of a rule that restricts inbound and outbound traffic",
                )
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_BASELINE)
                .weight(class(0))
                .action(ActionType::Block)
                .add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    fn install_permit_loopback(cx: &mut Cx<'_>) -> io::Result<()> {
        for (guid, layer, name) in [
            (
                FILTER_BASELINE_PERMIT_LOOPBACK_OUTBOUND_IPV4,
                Layer::ConnectV4,
                "Permit outbound connections on loopback (IPv4)",
            ),
            (
                FILTER_BASELINE_PERMIT_LOOPBACK_INBOUND_IPV4,
                Layer::AcceptV4,
                "Permit inbound connections on loopback (IPv4)",
            ),
            (
                FILTER_BASELINE_PERMIT_LOOPBACK_OUTBOUND_IPV6,
                Layer::ConnectV6,
                "Permit outbound connections on loopback (IPv6)",
            ),
            (
                FILTER_BASELINE_PERMIT_LOOPBACK_INBOUND_IPV6,
                Layer::AcceptV6,
                "Permit inbound connections on loopback (IPv6)",
            ),
        ] {
            let id = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description("This filter is part of a rule that permits all loopback traffic")
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_BASELINE)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(LoopbackConditionBuilder::new().build())
                .add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    fn install_permit_dhcp(cx: &mut Cx<'_>) -> io::Result<()> {
        // #1 Permit outbound DHCPv4 requests.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_IPV4)
            .name("Permit outbound DHCP requests (IPv4)")
            .description("This filter is part of a rule that permits DHCP client traffic")
            .provider(PROVIDER)
            .layer(Layer::ConnectV4)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_port_condition(ports::DHCP_V4_CLIENT_PORT))
            .condition(remote_ip_condition(IpAddr::V4(Ipv4Addr::BROADCAST)))
            .condition(remote_port_condition(ports::DHCP_V4_SERVER_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));

        // #2 Permit inbound DHCPv4 responses.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_INBOUND_RESPONSE_IPV4)
            .name("Permit inbound DHCP responses (IPv4)")
            .description("This filter is part of a rule that permits DHCP client traffic")
            .provider(PROVIDER)
            .layer(Layer::AcceptV4)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_port_condition(ports::DHCP_V4_CLIENT_PORT))
            .condition(remote_port_condition(ports::DHCP_V4_SERVER_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));

        let link_local = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0));
        let link_local_dhcp_multicast = IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2));
        let site_local_dhcp_multicast = IpAddr::V6(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 1, 3));

        // #3 Permit outbound DHCPv6 requests.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_IPV6)
            .name("Permit outbound DHCP requests (IPv6)")
            .description("This filter is part of a rule that permits DHCP client traffic")
            .provider(PROVIDER)
            .layer(Layer::ConnectV6)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_prefix_condition(link_local, 10))
            .condition(local_port_condition(ports::DHCP_V6_CLIENT_PORT))
            .condition(remote_ip_condition(link_local_dhcp_multicast))
            .condition(remote_ip_condition(site_local_dhcp_multicast))
            .condition(remote_port_condition(ports::DHCP_V6_SERVER_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));

        // #4 Permit inbound DHCPv6 responses.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_INBOUND_RESPONSE_IPV6)
            .name("Permit inbound DHCP responses (IPv6)")
            .description("This filter is part of a rule that permits DHCP client traffic")
            .provider(PROVIDER)
            .layer(Layer::AcceptV6)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_prefix_condition(link_local, 10))
            .condition(local_port_condition(ports::DHCP_V6_CLIENT_PORT))
            .condition(remote_prefix_condition(link_local, 10))
            .condition(remote_port_condition(ports::DHCP_V6_SERVER_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    fn install_permit_ndp(cx: &mut Cx<'_>) -> io::Result<()> {
        let link_local = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0));
        let link_local_router_multicast = IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 2));
        let solicited_node_multicast = IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 1, 0xff00, 0));

        /// The remote match of one NDP message: a single address, or a network.
        enum Remote {
            Single(IpAddr),
            Prefix(IpAddr, u8),
        }

        // (guid, layer, name, ICMP type, remote matches), one per message.
        let messages: [(wfp::Guid, Layer, &str, u8, Vec<Remote>); 7] = [
            // #1 Permit outbound router solicitation.
            (
                FILTER_BASELINE_PERMIT_NDP_OUTBOUND_ROUTER_SOLICITATION,
                Layer::ConnectV6,
                "Permit outbound NDP router solicitation",
                133,
                vec![Remote::Single(link_local_router_multicast)],
            ),
            // #2 Permit inbound router advertisement.
            (
                FILTER_BASELINE_PERMIT_NDP_INBOUND_ROUTER_ADVERTISEMENT,
                Layer::AcceptV6,
                "Permit inbound NDP router advertisement",
                134,
                vec![Remote::Prefix(link_local, 10)],
            ),
            // #3 Permit inbound redirect message.
            (
                FILTER_BASELINE_PERMIT_NDP_INBOUND_REDIRECT,
                Layer::AcceptV6,
                "Permit inbound NDP redirect",
                137,
                vec![Remote::Prefix(link_local, 10)],
            ),
            // #4 Permit outbound neighbor solicitation.
            (
                FILTER_BASELINE_PERMIT_NDP_OUTBOUND_NEIGHBOR_SOLICITATION,
                Layer::ConnectV6,
                "Permit outbound NDP neighbor solicitation",
                135,
                vec![
                    Remote::Prefix(solicited_node_multicast, 104),
                    Remote::Prefix(link_local, 10),
                ],
            ),
            // #5 Permit inbound neighbor solicitation.
            (
                FILTER_BASELINE_PERMIT_NDP_INBOUND_NEIGHBOR_SOLICITATION,
                Layer::AcceptV6,
                "Permit inbound NDP neighbor solicitation",
                135,
                vec![Remote::Prefix(link_local, 10)],
            ),
            // #6 Permit outbound neighbor advertisement.
            (
                FILTER_BASELINE_PERMIT_NDP_OUTBOUND_NEIGHBOR_ADVERTISEMENT,
                Layer::ConnectV6,
                "Permit outbound NDP neighbor advertisement",
                136,
                vec![Remote::Prefix(link_local, 10)],
            ),
            // #7 Permit inbound neighbor advertisement.
            (
                FILTER_BASELINE_PERMIT_NDP_INBOUND_NEIGHBOR_ADVERTISEMENT,
                Layer::AcceptV6,
                "Permit inbound NDP neighbor advertisement",
                136,
                vec![],
            ),
        ];

        for (guid, layer, name, icmp_type, remotes) in messages {
            let mut builder = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description(
                    "This filter is part of a rule that permits the most central parts of NDP",
                )
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_BASELINE)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(ProtocolConditionBuilder::icmpv6().build())
                .condition(IcmpConditionBuilder::r#type().equal(icmp_type).build())
                .condition(IcmpConditionBuilder::code().equal(0).build());

            for remote in remotes {
                builder = match remote {
                    Remote::Single(addr) => builder.condition(remote_ip_condition(addr)),
                    Remote::Prefix(addr, prefix) => {
                        builder.condition(remote_prefix_condition(addr, prefix))
                    }
                };
            }

            let id = builder.add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    fn install_permit_dhcp_server(cx: &mut Cx<'_>) -> io::Result<()> {
        // #1 Permit inbound DHCPv4 requests. Matches only the IPv4 broadcast,
        // the only supported extent.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_SERVER_INBOUND_REQUEST_IPV4)
            .name("Permit inbound DHCP requests (IPv4)")
            .description("This filter is part of a rule that permits DHCP server traffic")
            .provider(PROVIDER)
            .layer(Layer::AcceptV4)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_port_condition(ports::DHCP_V4_SERVER_PORT))
            .condition(local_ip_condition(IpAddr::V4(Ipv4Addr::BROADCAST)))
            .condition(remote_port_condition(ports::DHCP_V4_CLIENT_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));

        // #2 Permit outbound DHCPv4 responses.
        let id = FilterBuilder::default()
            .guid(FILTER_BASELINE_PERMIT_DHCP_SERVER_OUTBOUND_RESPONSE_IPV4)
            .name("Permit outbound DHCP responses (IPv4)")
            .description("This filter is part of a rule that permits DHCP server traffic")
            .provider(PROVIDER)
            .layer(Layer::ConnectV4)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(ProtocolConditionBuilder::udp().build())
            .condition(local_port_condition(ports::DHCP_V4_SERVER_PORT))
            .condition(remote_port_condition(ports::DHCP_V4_CLIENT_PORT))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    fn add_lan_traffic_filter(
        cx: &mut Cx<'_>,
        guid: wfp::Guid,
        layer: Layer,
        name: &str,
        description: &str,
        networks: impl Iterator<Item = (IpAddr, u8)>,
    ) -> io::Result<()> {
        let mut builder = FilterBuilder::default()
            .guid(guid)
            .name(name)
            .description(description)
            .provider(PROVIDER)
            .layer(layer)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(7))
            .action(ActionType::Permit);

        for (ip, prefix) in networks {
            builder = match ip {
                IpAddr::V4(ip) => builder.condition(
                    IpAddressConditionBuilder::remote()
                        .subnet_v4(ip, prefix)
                        .build(),
                ),
                IpAddr::V6(ip) => builder.condition(
                    IpAddressConditionBuilder::remote()
                        .subnet_v6(ip, prefix)
                        .build(),
                ),
            };
        }

        let id = builder.add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    fn install_permit_lan(cx: &mut Cx<'_>) -> io::Result<()> {
        // The LAN networks and multicast ranges of the platform, in order and
        // split by family.
        let (lan_v4, lan_v6) = split_networks(&ALLOWED_LAN_NETS);
        let (multicast_v4, multicast_v6) = split_networks(&ALLOWED_LAN_MULTICAST_NETS);

        // #1 Permit outbound connections on the LAN.
        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_OUTBOUND_IPV4,
            Layer::ConnectV4,
            "Permit outbound connections on LAN (IPv4)",
            "This filter is part of a rule that permits LAN traffic",
            lan_v4.iter().copied(),
        )?;

        // #2 Permit outbound multicast on the LAN.
        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_OUTBOUND_MULTICAST_IPV4,
            Layer::ConnectV4,
            "Permit outbound multicast on LAN (IPv4)",
            "This filter is part of a rule that permits LAN traffic",
            multicast_v4.iter().copied(),
        )?;

        // #3 Permit outbound connections on the LAN, IPv6.
        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_OUTBOUND_IPV6,
            Layer::ConnectV6,
            "Permit outbound connections on LAN (IPv6)",
            "This filter is part of a rule that permits LAN traffic",
            lan_v6.iter().copied(),
        )?;

        // #4 Permit outbound multicast on the LAN, IPv6.
        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_OUTBOUND_MULTICAST_IPV6,
            Layer::ConnectV6,
            "Permit outbound multicast on LAN (IPv6)",
            "This filter is part of a rule that permits LAN traffic",
            multicast_v6.iter().copied(),
        )?;
        Ok(())
    }

    fn install_permit_lan_service(cx: &mut Cx<'_>) -> io::Result<()> {
        let (lan_v4, lan_v6) = split_networks(&ALLOWED_LAN_NETS);

        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_SERVICE_INBOUND_IPV4,
            Layer::AcceptV4,
            "Permit inbound connections on LAN (IPv4)",
            "This filter is part of a rule that permits hosting services in a LAN environment",
            lan_v4.iter().copied(),
        )?;

        Self::add_lan_traffic_filter(
            cx,
            FILTER_BASELINE_PERMIT_LAN_SERVICE_INBOUND_IPV6,
            Layer::AcceptV6,
            "Permit inbound connections on LAN (IPv6)",
            "This filter is part of a rule that permits hosting services in a LAN environment",
            lan_v6.iter().copied(),
        )?;
        Ok(())
    }

    fn install_permit_dns(cx: &mut Cx<'_>) -> io::Result<()> {
        for (guid, layer, name) in [
            (
                FILTER_BASELINE_PERMIT_DNS_OUTBOUND_IPV4,
                Layer::ConnectV4,
                "Permit outbound connections to DNS server (IPv4)",
            ),
            (
                FILTER_BASELINE_PERMIT_DNS_OUTBOUND_IPV6,
                Layer::ConnectV6,
                "Permit outbound connections to DNS server (IPv6)",
            ),
        ] {
            let id = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description("This filter is part of a rule that permits outbound DNS")
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_BASELINE)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(remote_port_condition(ports::DNS_SERVER_PORT))
                .add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    /// The exit-relay access filters, then the endpoint filters.
    fn install_permit_vpn_tunnel(
        cx: &mut Cx<'_>,
        clients: &[PathBuf],
        alias: &str,
        endpoints: &Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    ) -> io::Result<()> {
        Self::add_exit_ip_rules(
            cx,
            clients,
            alias,
            exit_ip,
            Layer::ConnectV4,
            Layer::ConnectV6,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_EXIT_IP,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_BLOCK_EXIT_IP,
        )?;

        Self::add_tunnel_endpoint_rule(
            cx,
            alias,
            endpoints.as_ref().map(|(first, _)| first),
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_OUTBOUND_IPV4_1,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_OUTBOUND_IPV6_1,
            true,
        )?;

        if let Some((_, Some(second))) = endpoints {
            Self::add_tunnel_endpoint_rule(
                cx,
                alias,
                Some(second),
                FILTER_BASELINE_PERMIT_VPN_TUNNEL_OUTBOUND_IPV4_2,
                FILTER_BASELINE_PERMIT_VPN_TUNNEL_OUTBOUND_IPV6_2,
                true,
            )?;
        }
        Ok(())
    }

    /// The same as [`Self::install_permit_vpn_tunnel`], on the inbound layers.
    fn install_permit_vpn_tunnel_service(
        cx: &mut Cx<'_>,
        clients: &[PathBuf],
        alias: &str,
        endpoints: &Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    ) -> io::Result<()> {
        Self::add_exit_ip_rules(
            cx,
            clients,
            alias,
            exit_ip,
            Layer::AcceptV4,
            Layer::AcceptV6,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_EXIT_IP,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_BLOCK_EXIT_IP,
        )?;

        Self::add_tunnel_endpoint_rule(
            cx,
            alias,
            endpoints.as_ref().map(|(first, _)| first),
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_IPV4_1,
            FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_IPV6_1,
            false,
        )?;

        if let Some((_, Some(second))) = endpoints {
            Self::add_tunnel_endpoint_rule(
                cx,
                alias,
                Some(second),
                FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_IPV4_2,
                FILTER_BASELINE_PERMIT_VPN_TUNNEL_SERVICE_IPV6_2,
                false,
            )?;
        }
        Ok(())
    }

    /// Permit the relay clients to reach the exit relay, and block everyone
    /// else from it.
    ///
    /// The filter names say "inbound" even on the outbound layers.
    #[expect(clippy::too_many_arguments)]
    fn add_exit_ip_rules(
        cx: &mut Cx<'_>,
        clients: &[PathBuf],
        alias: &str,
        exit_ip: Option<IpAddr>,
        layer_v4: Layer,
        layer_v6: Layer,
        permit_guid: wfp::Guid,
        block_guid: wfp::Guid,
    ) -> io::Result<()> {
        let Some(exit_ip) = exit_ip else {
            return Ok(());
        };
        // Without relay clients, everyone may reach the exit relay.
        if clients.is_empty() {
            return Ok(());
        }

        let layer = match exit_ip {
            IpAddr::V4(_) => layer_v4,
            IpAddr::V6(_) => layer_v6,
        };

        let builder = FilterBuilder::default()
            .guid(permit_guid)
            .name("Permit inbound exit relay connections on tunnel interface")
            .description(
                "This filter is part of a rule that allows exit IP traffic from select clients",
            )
            .provider(PROVIDER)
            .layer(layer)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(15))
            .action(ActionType::Permit)
            .condition(interface_alias_condition(alias)?)
            .condition(remote_ip_condition(exit_ip));
        let id = app_id_conditions(builder, clients)?.add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));

        let id = FilterBuilder::default()
            .guid(block_guid)
            .name("Block inbound exit relay connections on tunnel interface")
            .description(
                "This filter is part of a rule that blocks exit IP traffic from unexpected clients",
            )
            .provider(PROVIDER)
            .layer(layer)
            .sublayer(SUBLAYER_BASELINE)
            .weight(class(15))
            .action(ActionType::Block)
            .condition(interface_alias_condition(alias)?)
            .condition(remote_ip_condition(exit_ip))
            .add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    /// Permit traffic inside the tunnel, to one specific endpoint or to
    /// anywhere.
    fn add_tunnel_endpoint_rule(
        cx: &mut Cx<'_>,
        alias: &str,
        endpoint: Option<&RelayEndpoint>,
        ipv4_guid: wfp::Guid,
        ipv6_guid: wfp::Guid,
        outbound: bool,
    ) -> io::Result<()> {
        let (layer_v4, layer_v6, name, description) = if outbound {
            (
                Layer::ConnectV4,
                Layer::ConnectV6,
                "Permit outbound connections on tunnel interface",
                "This filter is part of a rule that permits communications inside the VPN tunnel",
            )
        } else {
            (
                Layer::AcceptV4,
                Layer::AcceptV6,
                "Permit inbound connections on tunnel interface",
                "This filter is part of a rule that permits hosting services that listen on the tunnel interface",
            )
        };

        let mut add = |guid, layer: Layer, ip_version: &str| -> io::Result<()> {
            let mut builder = FilterBuilder::default()
                .guid(guid)
                .name(format!("{name} ({ip_version})"))
                .description(description)
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_BASELINE)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(interface_alias_condition(alias)?);

            if let Some(endpoint) = endpoint {
                builder = builder
                    .condition(remote_ip_condition(endpoint.ip))
                    .condition(remote_port_condition(endpoint.port))
                    .condition(endpoint.protocol.condition());
            }

            let id = builder.add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
            Ok(())
        };

        let should_add_v4 = endpoint.is_none_or(|endpoint| endpoint.ip.is_ipv4());
        let should_add_v6 = endpoint.is_none_or(|endpoint| endpoint.ip.is_ipv6());

        if should_add_v4 {
            add(ipv4_guid, layer_v4, "IPv4")?;
        }
        if should_add_v6 {
            add(ipv6_guid, layer_v6, "IPv6")?;
        }
        Ok(())
    }

    fn install_dns_block_all(cx: &mut Cx<'_>) -> io::Result<()> {
        for (guid, layer, name) in [
            (
                FILTER_DNS_BLOCK_ALL_OUTBOUND_IPV4,
                Layer::ConnectV4,
                "Block outbound DNS (IPv4)",
            ),
            (
                FILTER_DNS_BLOCK_ALL_OUTBOUND_IPV6,
                Layer::ConnectV6,
                "Block outbound DNS (IPv6)",
            ),
        ] {
            let id = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description("This filter is part of a rule that blocks DNS requests")
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_DNS)
                .weight(class(0))
                .action(ActionType::Block)
                .condition(remote_port_condition(ports::DNS_SERVER_PORT))
                .add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    fn install_dns_permit_loopback(cx: &mut Cx<'_>) -> io::Result<()> {
        for (guid, layer, name) in [
            (
                FILTER_DNS_PERMIT_LOOPBACK_OUTBOUND_IPV4,
                Layer::ConnectV4,
                "Permit loopback DNS traffic (IPv4)",
            ),
            (
                FILTER_DNS_PERMIT_LOOPBACK_OUTBOUND_IPV6,
                Layer::ConnectV6,
                "Permit loopback DNS traffic (IPv6)",
            ),
        ] {
            let id = FilterBuilder::default()
                .guid(guid)
                .name(name)
                .description("This filter is part of a rule that permits loopback DNS traffic")
                .provider(PROVIDER)
                .layer(layer)
                .sublayer(SUBLAYER_DNS)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(LoopbackConditionBuilder::new().build())
                .condition(remote_port_condition(ports::DNS_SERVER_PORT))
                .add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }
        Ok(())
    }

    fn install_dns_permit_tunnel(cx: &mut Cx<'_>, alias: &str, hosts: &[IpAddr]) -> io::Result<()> {
        let (hosts_v4, hosts_v6) = split_ips(hosts);

        if !hosts_v4.is_empty() {
            let mut builder = FilterBuilder::default()
                .guid(FILTER_DNS_PERMIT_TUNNEL_OUTBOUND_IPV4)
                .name("Permit selected DNS traffic inside tunnel (IPv4)")
                .description(
                    "This filter is part of a rule that permits DNS traffic inside the VPN tunnel",
                )
                .provider(PROVIDER)
                .layer(Layer::ConnectV4)
                .sublayer(SUBLAYER_DNS)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(remote_port_condition(ports::DNS_SERVER_PORT))
                .condition(interface_alias_condition(alias)?);

            for host in &hosts_v4 {
                builder = builder.condition(remote_ip_condition(IpAddr::V4(*host)));
            }

            let id = builder.add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }

        if hosts_v6.is_empty() {
            return Ok(());
        }

        let mut builder = FilterBuilder::default()
            .guid(FILTER_DNS_PERMIT_TUNNEL_OUTBOUND_IPV6)
            .name("Permit selected DNS traffic inside tunnel (IPv6)")
            .description(
                "This filter is part of a rule that permits DNS traffic inside the VPN tunnel",
            )
            .provider(PROVIDER)
            .layer(Layer::ConnectV6)
            .sublayer(SUBLAYER_DNS)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(remote_port_condition(ports::DNS_SERVER_PORT))
            .condition(interface_alias_condition(alias)?);

        for host in &hosts_v6 {
            builder = builder.condition(remote_ip_condition(IpAddr::V6(*host)));
        }

        let id = builder.add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    fn install_dns_permit_non_tunnel(
        cx: &mut Cx<'_>,
        alias: &Option<String>,
        hosts: &[IpAddr],
    ) -> io::Result<()> {
        let (hosts_v4, hosts_v6) = split_ips(hosts);

        if !hosts_v4.is_empty() {
            let mut builder = FilterBuilder::default()
                .guid(FILTER_DNS_PERMIT_NON_TUNNEL_OUTBOUND_IPV4)
                .name("Permit selected non-tunnel DNS traffic (IPv4)")
                .description("This filter is part of a rule that permits non-tunnel DNS traffic")
                .provider(PROVIDER)
                .layer(Layer::ConnectV4)
                .sublayer(SUBLAYER_DNS)
                .weight(class(7))
                .action(ActionType::Permit)
                .condition(remote_port_condition(ports::DNS_SERVER_PORT));

            for host in &hosts_v4 {
                builder = builder.condition(remote_ip_condition(IpAddr::V4(*host)));
            }
            if let Some(alias) = alias {
                builder = builder.condition(interface_alias_not_condition(alias)?);
            }

            let id = builder.add(cx.transaction)?;
            cx.recorder.record(Record::Filter(id));
        }

        if hosts_v6.is_empty() {
            return Ok(());
        }

        let mut builder = FilterBuilder::default()
            .guid(FILTER_DNS_PERMIT_NON_TUNNEL_OUTBOUND_IPV6)
            .name("Permit selected non-tunnel DNS traffic (IPv6)")
            .description("This filter is part of a rule that permits non-tunnel DNS traffic")
            .provider(PROVIDER)
            .layer(Layer::ConnectV6)
            .sublayer(SUBLAYER_DNS)
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(remote_port_condition(ports::DNS_SERVER_PORT));

        for host in &hosts_v6 {
            builder = builder.condition(remote_ip_condition(IpAddr::V6(*host)));
        }
        if let Some(alias) = alias {
            builder = builder.condition(interface_alias_not_condition(alias)?);
        }

        let id = builder.add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }

    fn install_permit_endpoint(
        cx: &mut Cx<'_>,
        endpoint: &RelayEndpoint,
        clients: &[PathBuf],
        sublayer: Sublayer,
    ) -> io::Result<()> {
        let layer = match endpoint.ip {
            IpAddr::V4(_) => Layer::ConnectV4,
            IpAddr::V6(_) => Layer::ConnectV6,
        };

        let builder = FilterBuilder::default()
            .name("Permit outbound connections to an endpoint")
            .description(
                "This filter is part of a rule that permits communication with an endpoint",
            )
            .provider(PROVIDER)
            .layer(layer)
            .sublayer(sublayer.guid())
            .weight(class(7))
            .action(ActionType::Permit)
            .condition(remote_ip_condition(endpoint.ip))
            .condition(remote_port_condition(endpoint.port))
            .condition(endpoint.protocol.condition());

        let id = app_id_conditions(builder, clients)?.add(cx.transaction)?;
        cx.recorder.record(Record::Filter(id));
        Ok(())
    }
}

/// The filter weight range `class`, from 0 to 15.
fn class(class: u8) -> FilterWeight {
    FilterWeight::Range(WeightRange::try_from(class).expect("weight class 0..=15"))
}

fn remote_ip_condition(ip: IpAddr) -> wfp::Condition {
    match ip {
        IpAddr::V4(ip) => IpAddressConditionBuilder::remote().host_v4(ip).build(),
        IpAddr::V6(ip) => IpAddressConditionBuilder::remote().host_v6(ip).build(),
    }
}

fn local_ip_condition(ip: IpAddr) -> wfp::Condition {
    match ip {
        IpAddr::V4(ip) => IpAddressConditionBuilder::local().host_v4(ip).build(),
        IpAddr::V6(ip) => IpAddressConditionBuilder::local().host_v6(ip).build(),
    }
}

fn remote_port_condition(port: u16) -> wfp::Condition {
    PortConditionBuilder::remote().equal(port).build()
}

fn local_port_condition(port: u16) -> wfp::Condition {
    PortConditionBuilder::local().equal(port).build()
}

fn interface_alias_condition(alias: &str) -> io::Result<wfp::Condition> {
    Ok(InterfaceConditionBuilder::local().alias(alias)?.build())
}

fn interface_alias_not_condition(alias: &str) -> io::Result<wfp::Condition> {
    Ok(InterfaceConditionBuilder::local()
        .alias(alias)?
        .not_equal()
        .build())
}

/// Match a remote network prefix.
fn remote_prefix_condition(addr: IpAddr, prefix: u8) -> wfp::Condition {
    match addr {
        IpAddr::V4(ip) => IpAddressConditionBuilder::remote()
            .subnet_v4(ip, prefix)
            .build(),
        IpAddr::V6(ip) => IpAddressConditionBuilder::remote()
            .subnet_v6(ip, prefix)
            .build(),
    }
}

/// Match a local network prefix.
fn local_prefix_condition(addr: IpAddr, prefix: u8) -> wfp::Condition {
    match addr {
        IpAddr::V4(ip) => IpAddressConditionBuilder::local()
            .subnet_v4(ip, prefix)
            .build(),
        IpAddr::V6(ip) => IpAddressConditionBuilder::local()
            .subnet_v6(ip, prefix)
            .build(),
    }
}

fn app_id_conditions(
    builder: FilterBuilder<wfp::FilterBuilderHasName, wfp::FilterBuilderHasAction>,
    clients: &[PathBuf],
) -> io::Result<FilterBuilder<wfp::FilterBuilderHasName, wfp::FilterBuilderHasAction>> {
    let mut builder = builder;
    for client in clients {
        builder = builder.condition(AppIdConditionBuilder::new().equal(client)?.build());
    }
    Ok(builder)
}

/// A network address and its prefix length.
type Net = (IpAddr, u8);

fn split_networks(networks: &[ipnetwork::IpNetwork]) -> (Vec<Net>, Vec<Net>) {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    for network in networks {
        let net = (network.ip(), network.prefix());
        match net.0 {
            IpAddr::V4(_) => v4.push(net),
            IpAddr::V6(_) => v6.push(net),
        }
    }
    (v4, v6)
}

fn split_ips(hosts: &[IpAddr]) -> (Vec<Ipv4Addr>, Vec<Ipv6Addr>) {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    for host in hosts {
        match host {
            IpAddr::V4(ip) => v4.push(*ip),
            IpAddr::V6(ip) => v6.push(*ip),
        }
    }
    (v4, v6)
}
