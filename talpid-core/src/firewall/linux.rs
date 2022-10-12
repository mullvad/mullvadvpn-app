use super::{FirewallArguments, FirewallPolicy};
use crate::{split_tunnel, tunnel};
use ipnetwork::IpNetwork;
use lazy_static::lazy_static;
use libc;
use nftnl::{
    self,
    expr::{self, IcmpCode, Payload, RejectionType, Verdict},
    nft_expr, table, Batch, Chain, FinalizedBatch, ProtoFamily, Rule, Table,
};
use std::{
    env,
    ffi::{CStr, CString},
    fs, io,
    net::{IpAddr, Ipv4Addr},
};
use talpid_types::net::{AllowedTunnelTraffic, Endpoint, TransportProtocol};

/// Priority for rules that tag split tunneling packets. Equals NF_IP_PRI_MANGLE.
const MANGLE_CHAIN_PRIORITY: i32 = libc::NF_IP_PRI_MANGLE;
const PREROUTING_CHAIN_PRIORITY: i32 = libc::NF_IP_PRI_CONNTRACK + 1;
const PROC_SYS_NET_IPV4_CONF_SRC_VALID_MARK: &str = "/proc/sys/net/ipv4/conf/all/src_valid_mark";

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen when interacting with Linux netfilter.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Unable to open netlink socket to netfilter.
    #[error(display = "Unable to open netlink socket to netfilter")]
    NetlinkOpenError(#[error(source)] io::Error),

    /// Unable to send netlink command to netfilter.
    #[error(display = "Unable to send netlink command to netfilter")]
    NetlinkSendError(#[error(source)] io::Error),

    /// Error while reading from netlink socket.
    #[error(display = "Error while reading from netlink socket")]
    NetlinkRecvError(#[error(source)] io::Error),

    /// Error while processing an incoming netlink message.
    #[error(display = "Error while processing an incoming netlink message")]
    ProcessNetlinkError(#[error(source)] io::Error),

    /// Failed to verify that our tables are set. Probably means that
    /// it's the host that does not support nftables properly.
    #[error(display = "Failed to set firewall rules")]
    NetfilterTableNotSetError,

    /// Unable to translate network interface name into index.
    #[error(
        display = "Unable to translate network interface name \"{}\" into index",
        _0
    )]
    LookupIfaceIndexError(String, #[error(source)] crate::linux::IfaceIndexLookupError),
}

lazy_static! {
    /// TODO(linus): This crate is not supposed to be Mullvad-aware. So at some point this should be
    /// replaced by allowing the table name to be configured from the public API of this crate.
    static ref TABLE_NAME: CString = CString::new("mullvad").unwrap();
    static ref IN_CHAIN_NAME: CString = CString::new("input").unwrap();
    static ref OUT_CHAIN_NAME: CString = CString::new("output").unwrap();
    static ref FORWARD_CHAIN_NAME: CString = CString::new("forward").unwrap();
    static ref PREROUTING_CHAIN_NAME: CString = CString::new("prerouting").unwrap();

    /// We need two separate tables for compatibility with older kernels (holds true for kernel
    /// version 4.19 but not 5.6), where the base filter type may not be `nftnl::ChainType::Route`
    /// or `nftnl::ChainType::Nat` for inet tables.
    static ref MANGLE_TABLE_NAME_V4: CString = CString::new("mullvadmangle4").unwrap();
    static ref MANGLE_TABLE_NAME_V6: CString = CString::new("mullvadmangle6").unwrap();
    static ref MANGLE_CHAIN_NAME: CString = CString::new("mangle").unwrap();
    static ref NAT_CHAIN_NAME: CString = CString::new("nat").unwrap();

    /// Allows controlling whether firewall rules should have packet counters or not from an env
    /// variable. Useful for debugging the rules.
    static ref ADD_COUNTERS: bool = env::var("TALPID_FIREWALL_DEBUG")
        .map(|v| v != "0")
        .unwrap_or(false);

    static ref DONT_SET_SRC_VALID_MARK: bool = env::var("TALPID_FIREWALL_DONT_SET_SRC_VALID_MARK")
        .map(|v| v != "0")
        .unwrap_or(false);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    In,
    Out,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum End {
    Src,
    Dst,
}

/// The Linux implementation for the firewall and DNS.
pub struct Firewall {
    fwmark: u32,
}

struct FirewallTables {
    main: Table,
    mangle_v4: Table,
    mangle_v6: Table,
}

impl Firewall {
    pub fn from_args(args: FirewallArguments) -> Result<Self> {
        Firewall::new(args.fwmark)
    }

    pub fn new(fwmark: u32) -> Result<Self> {
        Ok(Firewall { fwmark })
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<()> {
        let tables = FirewallTables {
            main: Table::new(&*TABLE_NAME, ProtoFamily::Inet),
            mangle_v4: Table::new(&*MANGLE_TABLE_NAME_V4, ProtoFamily::Ipv4),
            mangle_v6: Table::new(&*MANGLE_TABLE_NAME_V6, ProtoFamily::Ipv6),
        };
        let batch = PolicyBatch::new(&tables).finalize(&policy, self.fwmark)?;
        Self::send_and_process(&batch)?;
        Self::apply_kernel_config(&policy);
        self.verify_tables(&[&TABLE_NAME, &MANGLE_TABLE_NAME_V4, &MANGLE_TABLE_NAME_V6])
    }

    pub fn reset_policy(&mut self) -> Result<()> {
        let tables = [
            Table::new(&*TABLE_NAME, ProtoFamily::Inet),
            Table::new(&*MANGLE_TABLE_NAME_V4, ProtoFamily::Ipv4),
            Table::new(&*MANGLE_TABLE_NAME_V6, ProtoFamily::Ipv6),
        ];
        let mut batch = Batch::new();
        for table in &tables {
            // Our batch will add and remove the table even though the goal is just to remove
            // it. This because only removing it throws a strange error if the
            // table does not exist.
            batch.add(table, nftnl::MsgType::Add);
            batch.add(table, nftnl::MsgType::Del);
        }
        let batch = batch.finalize();
        log::debug!("Removing table and chain from netfilter");
        Self::send_and_process(&batch)?;
        Ok(())
    }

    fn apply_kernel_config(policy: &FirewallPolicy) {
        if *DONT_SET_SRC_VALID_MARK {
            log::debug!("Not setting src_valid_mark");
            return;
        }

        if let FirewallPolicy::Connecting { .. } = policy {
            if let Err(err) = set_src_valid_mark_sysctl() {
                log::error!("Failed to apply src_valid_mark: {}", err);
            }
        }
    }

    fn send_and_process(batch: &FinalizedBatch) -> Result<()> {
        let socket = mnl::Socket::new(mnl::Bus::Netfilter).map_err(Error::NetlinkOpenError)?;
        socket.send_all(batch).map_err(Error::NetlinkSendError)?;

        let portid = socket.portid();
        let mut buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];

        let seq = 0;
        while let Some(message) = Self::socket_recv(&socket, &mut buffer[..])? {
            match mnl::cb_run(message, seq, portid).map_err(Error::ProcessNetlinkError)? {
                mnl::CbResult::Stop => {
                    log::trace!("cb_run STOP");
                    break;
                }
                mnl::CbResult::Ok => log::trace!("cb_run OK"),
            };
        }
        Ok(())
    }

    fn verify_tables(&self, expected_tables: &[&CStr]) -> Result<()> {
        let socket = mnl::Socket::new(mnl::Bus::Netfilter).map_err(Error::NetlinkOpenError)?;
        let portid = socket.portid();
        let seq = 0;

        let get_tables_msg = table::get_tables_nlmsg(seq);
        socket
            .send(&get_tables_msg)
            .map_err(Error::NetlinkSendError)?;

        let mut table_set = std::collections::HashSet::new();
        let mut msg_buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];

        while let Some(message) = Self::socket_recv(&socket, &mut msg_buffer)? {
            match mnl::cb_run2(message, seq, portid, table::get_tables_cb, &mut table_set)
                .map_err(Error::ProcessNetlinkError)?
            {
                mnl::CbResult::Stop => {
                    log::trace!("cb_run STOP");
                    break;
                }
                mnl::CbResult::Ok => log::trace!("cb_run OK"),
            }
        }

        for expected_table in expected_tables {
            if !table_set.contains(*expected_table) {
                log::error!(
                    "Expected '{}' netfilter table to be set, but it is not",
                    expected_table.to_string_lossy()
                );
                return Err(Error::NetfilterTableNotSetError);
            }
        }
        Ok(())
    }

    fn socket_recv<'a>(socket: &mnl::Socket, buf: &'a mut [u8]) -> Result<Option<&'a [u8]>> {
        let ret = socket.recv(buf).map_err(Error::NetlinkRecvError)?;
        log::trace!("Read {} bytes from netlink", ret);
        if ret > 0 {
            Ok(Some(&buf[..ret]))
        } else {
            Ok(None)
        }
    }
}

struct PolicyBatch<'a> {
    batch: Batch,
    in_chain: Chain<'a>,
    out_chain: Chain<'a>,
    forward_chain: Chain<'a>,
    prerouting_chain: Chain<'a>,
    mangle_chain_v4: Chain<'a>,
    mangle_chain_v6: Chain<'a>,
    nat_chain_v4: Chain<'a>,
    nat_chain_v6: Chain<'a>,
}

impl<'a> PolicyBatch<'a> {
    /// Bootstrap a new nftnl message batch object and add the initial messages creating the
    /// table and chains.
    pub fn new(tables: &'a FirewallTables) -> Self {
        let mut batch = Batch::new();
        let mut prerouting_chain = Chain::new(&*PREROUTING_CHAIN_NAME, &tables.main);
        prerouting_chain.set_hook(nftnl::Hook::PreRouting, PREROUTING_CHAIN_PRIORITY);
        prerouting_chain.set_type(nftnl::ChainType::Filter);

        let mut out_chain = Chain::new(&*OUT_CHAIN_NAME, &tables.main);
        out_chain.set_hook(nftnl::Hook::Out, 0);
        out_chain.set_policy(nftnl::Policy::Drop);

        let mut in_chain = Chain::new(&*IN_CHAIN_NAME, &tables.main);
        in_chain.set_hook(nftnl::Hook::In, 0);
        in_chain.set_policy(nftnl::Policy::Drop);

        let mut forward_chain = Chain::new(&*FORWARD_CHAIN_NAME, &tables.main);
        forward_chain.set_hook(nftnl::Hook::Forward, 0);
        forward_chain.set_policy(nftnl::Policy::Drop);

        Self::flush_table(&mut batch, &tables.main);
        batch.add(&prerouting_chain, nftnl::MsgType::Add);
        batch.add(&out_chain, nftnl::MsgType::Add);
        batch.add(&in_chain, nftnl::MsgType::Add);
        batch.add(&forward_chain, nftnl::MsgType::Add);

        Self::flush_table(&mut batch, &tables.mangle_v4);
        Self::flush_table(&mut batch, &tables.mangle_v6);

        let mut add_mangle_chain = |table| {
            let mut chain = Chain::new(&*MANGLE_CHAIN_NAME, table);
            chain.set_hook(nftnl::Hook::Out, MANGLE_CHAIN_PRIORITY);
            chain.set_type(nftnl::ChainType::Route);
            chain.set_policy(nftnl::Policy::Accept);
            batch.add(&chain, nftnl::MsgType::Add);

            chain
        };
        let mangle_chain_v4 = add_mangle_chain(&tables.mangle_v4);
        let mangle_chain_v6 = add_mangle_chain(&tables.mangle_v6);

        let mut add_nat_chain = |table| {
            let mut chain = Chain::new(&*NAT_CHAIN_NAME, table);
            chain.set_hook(nftnl::Hook::PostRouting, libc::NF_IP_PRI_NAT_SRC);
            chain.set_type(nftnl::ChainType::Nat);
            chain.set_policy(nftnl::Policy::Accept);
            batch.add(&chain, nftnl::MsgType::Add);

            chain
        };
        let nat_chain_v4 = add_nat_chain(&tables.mangle_v4);
        let nat_chain_v6 = add_nat_chain(&tables.mangle_v6);

        PolicyBatch {
            batch,
            in_chain,
            out_chain,
            forward_chain,
            prerouting_chain,
            mangle_chain_v4,
            mangle_chain_v6,
            nat_chain_v4,
            nat_chain_v6,
        }
    }

    /// Creates the table if it does not exist and clears it otherwise.
    fn flush_table(batch: &mut Batch, table: &'a Table) {
        batch.add(table, nftnl::MsgType::Add);
        batch.add(table, nftnl::MsgType::Del);
        batch.add(table, nftnl::MsgType::Add);
    }

    /// Finalize the nftnl message batch by adding every firewall rule needed to satisfy the given
    /// policy.
    pub fn finalize(mut self, policy: &FirewallPolicy, fwmark: u32) -> Result<FinalizedBatch> {
        self.add_loopback_rules()?;
        self.add_split_tunneling_rules(policy, fwmark)?;
        self.add_dhcp_client_rules();
        self.add_ndp_rules();
        self.add_policy_specific_rules(policy, fwmark)?;

        Ok(self.batch.finalize())
    }

    fn add_split_tunneling_rules(&mut self, policy: &FirewallPolicy, fwmark: u32) -> Result<()> {
        // Send select DNS requests in the tunnel
        if let FirewallPolicy::Connected {
            tunnel,
            dns_servers,
            ..
        } = policy
        {
            for server in dns_servers
                .iter()
                .filter(|server| !is_local_dns_address(tunnel, server))
            {
                let chain = if server.is_ipv4() {
                    &self.mangle_chain_v4
                } else {
                    &self.mangle_chain_v6
                };
                let allow_rule = allow_tunnel_dns_rule(
                    chain,
                    &tunnel.interface,
                    TransportProtocol::Udp,
                    *server,
                )?;
                self.batch.add(&allow_rule, nftnl::MsgType::Add);
                let allow_rule = allow_tunnel_dns_rule(
                    chain,
                    &tunnel.interface,
                    TransportProtocol::Tcp,
                    *server,
                )?;
                self.batch.add(&allow_rule, nftnl::MsgType::Add);
            }
        }

        let mangle_chains = [&self.mangle_chain_v4, &self.mangle_chain_v6];
        for chain in &mangle_chains {
            let mut rule = Rule::new(chain);
            rule.add_expr(&nft_expr!(meta cgroup));
            rule.add_expr(&nft_expr!(cmp == split_tunnel::NET_CLS_CLASSID));
            rule.add_expr(&nft_expr!(immediate data split_tunnel::MARK));
            rule.add_expr(&nft_expr!(ct mark set));
            rule.add_expr(&nft_expr!(immediate data fwmark));
            rule.add_expr(&nft_expr!(meta mark set));
            self.batch.add(&rule, nftnl::MsgType::Add);
        }

        for chain in &[&self.in_chain, &self.out_chain] {
            let mut rule = Rule::new(chain);
            rule.add_expr(&nft_expr!(ct mark));
            rule.add_expr(&nft_expr!(cmp == split_tunnel::MARK));
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }

        let nat_chains = [&self.nat_chain_v4, &self.nat_chain_v6];
        for chain in &nat_chains {
            // Block remaining marked outgoing in-tunnel traffic
            if let FirewallPolicy::Connected { tunnel, .. } = policy {
                let mut block_tunnel_rule = Rule::new(chain);
                check_iface(&mut block_tunnel_rule, Direction::Out, &tunnel.interface)?;
                block_tunnel_rule.add_expr(&nft_expr!(ct mark));
                block_tunnel_rule.add_expr(&nft_expr!(cmp == split_tunnel::MARK));
                add_verdict(&mut block_tunnel_rule, &Verdict::Drop);
                self.batch.add(&block_tunnel_rule, nftnl::MsgType::Add);
            }

            // Replace source IP address in rerouted packets.
            // Don't masquerade packets on the loopback device.
            let mut rule = Rule::new(chain);

            let iface_index = crate::linux::iface_index("lo")
                .map_err(|e| Error::LookupIfaceIndexError("lo".to_string(), e))?;
            rule.add_expr(&nft_expr!(meta oif));
            rule.add_expr(&nft_expr!(cmp != iface_index));

            rule.add_expr(&nft_expr!(ct mark));
            rule.add_expr(&nft_expr!(cmp == split_tunnel::MARK));

            rule.add_expr(&nft_expr!(masquerade));
            if *ADD_COUNTERS {
                rule.add_expr(&nft_expr!(counter));
            }
            self.batch.add(&rule, nftnl::MsgType::Add);
        }

        // Route incoming traffic correctly to prevent strict rpf from rejecting packets
        // for excluded processes
        if let FirewallPolicy::Connected { tunnel, .. } = policy {
            let mut prerouting_rule = Rule::new(&self.prerouting_chain);
            check_not_iface(&mut prerouting_rule, Direction::In, &tunnel.interface)?;
            prerouting_rule.add_expr(&nft_expr!(ct mark));
            prerouting_rule.add_expr(&nft_expr!(cmp == split_tunnel::MARK));
            prerouting_rule.add_expr(&nft_expr!(immediate data fwmark));
            prerouting_rule.add_expr(&nft_expr!(meta mark set));
            if *ADD_COUNTERS {
                prerouting_rule.add_expr(&nft_expr!(counter));
            }
            self.batch.add(&prerouting_rule, nftnl::MsgType::Add);
        }

        Ok(())
    }

    fn add_loopback_rules(&mut self) -> Result<()> {
        const LOOPBACK_IFACE_NAME: &str = "lo";
        self.batch.add(
            &allow_interface_rule(&self.out_chain, Direction::Out, LOOPBACK_IFACE_NAME)?,
            nftnl::MsgType::Add,
        );
        self.batch.add(
            &allow_interface_rule(&self.in_chain, Direction::In, LOOPBACK_IFACE_NAME)?,
            nftnl::MsgType::Add,
        );
        Ok(())
    }

    fn add_dhcp_client_rules(&mut self) {
        use self::TransportProtocol::Udp;
        // Outgoing DHCPv4 request
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut out_v4 = Rule::new(chain);
            check_port(&mut out_v4, Udp, End::Src, super::DHCPV4_CLIENT_PORT);
            check_ip(&mut out_v4, End::Dst, IpAddr::V4(Ipv4Addr::BROADCAST));
            check_port(&mut out_v4, Udp, End::Dst, super::DHCPV4_SERVER_PORT);
            add_verdict(&mut out_v4, &Verdict::Accept);
            self.batch.add(&out_v4, nftnl::MsgType::Add);
        }
        // Incoming DHCPv4 response
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut in_v4 = Rule::new(chain);
            check_port(&mut in_v4, Udp, End::Src, super::DHCPV4_SERVER_PORT);
            check_port(&mut in_v4, Udp, End::Dst, super::DHCPV4_CLIENT_PORT);
            add_verdict(&mut in_v4, &Verdict::Accept);
            self.batch.add(&in_v4, nftnl::MsgType::Add);
        }

        for chain in &[&self.out_chain, &self.forward_chain] {
            for dhcpv6_server in &*super::DHCPV6_SERVER_ADDRS {
                let mut out_v6 = Rule::new(chain);
                check_net(&mut out_v6, End::Src, *super::IPV6_LINK_LOCAL);
                check_port(&mut out_v6, Udp, End::Src, super::DHCPV6_CLIENT_PORT);
                check_ip(&mut out_v6, End::Dst, *dhcpv6_server);
                check_port(&mut out_v6, Udp, End::Dst, super::DHCPV6_SERVER_PORT);
                add_verdict(&mut out_v6, &Verdict::Accept);
                self.batch.add(&out_v6, nftnl::MsgType::Add);
            }
        }
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut in_v6 = Rule::new(chain);
            check_net(&mut in_v6, End::Src, *super::IPV6_LINK_LOCAL);
            check_port(&mut in_v6, Udp, End::Src, super::DHCPV6_SERVER_PORT);
            check_net(&mut in_v6, End::Dst, *super::IPV6_LINK_LOCAL);
            check_port(&mut in_v6, Udp, End::Dst, super::DHCPV6_CLIENT_PORT);
            add_verdict(&mut in_v6, &Verdict::Accept);
            self.batch.add(&in_v6, nftnl::MsgType::Add);
        }
    }

    fn add_ndp_rules(&mut self) {
        // Outgoing Router solicitation (part of NDP)
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_ip(
                &mut rule,
                End::Dst,
                *super::ROUTER_SOLICITATION_OUT_DST_ADDR,
            );
            check_icmpv6(&mut rule, 133, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Incoming Router advertisement (part of NDP)
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Src, *super::IPV6_LINK_LOCAL);
            check_icmpv6(&mut rule, 134, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Incoming Redirect (part of NDP)
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Src, *super::IPV6_LINK_LOCAL);
            check_icmpv6(&mut rule, 137, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Outgoing Neighbor solicitation (part of NDP)
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Dst, *super::SOLICITED_NODE_MULTICAST);
            check_icmpv6(&mut rule, 135, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Dst, *super::IPV6_LINK_LOCAL);
            check_icmpv6(&mut rule, 135, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Incoming Neighbor solicitation (part of NDP)
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Src, *super::IPV6_LINK_LOCAL);
            check_icmpv6(&mut rule, 135, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Outgoing Neighbor advertisement (part of NDP)
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_net(&mut rule, End::Dst, *super::IPV6_LINK_LOCAL);
            check_icmpv6(&mut rule, 136, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        // Incoming Neighbor advertisement (part of NDP)
        for chain in &[&self.in_chain, &self.forward_chain] {
            let mut rule = Rule::new(chain);
            check_icmpv6(&mut rule, 136, 0);
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
    }

    fn add_policy_specific_rules(&mut self, policy: &FirewallPolicy, fwmark: u32) -> Result<()> {
        let allow_lan = match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
            } => {
                self.add_allow_tunnel_endpoint_rules(peer_endpoint, fwmark);
                self.add_allow_endpoint_rules(&allowed_endpoint.endpoint);

                // Important to block DNS after allow relay rule (so the relay can operate
                // over port 53) but before allow LAN (so DNS does not leak to the LAN)
                self.add_drop_dns_rule();

                if let Some(tunnel) = tunnel {
                    match allowed_tunnel_traffic {
                        AllowedTunnelTraffic::All => {
                            self.add_allow_tunnel_rules(&tunnel.interface)?;
                        }
                        AllowedTunnelTraffic::None => (),
                        AllowedTunnelTraffic::Only(endpoint) => {
                            self.add_allow_in_tunnel_endpoint_rules(&tunnel.interface, endpoint)?;
                        }
                    }
                    if *allow_lan {
                        self.add_block_cve_2019_14899(tunnel);
                    }
                }
                *allow_lan
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
                dns_servers,
            } => {
                self.add_allow_tunnel_endpoint_rules(peer_endpoint, fwmark);
                self.add_allow_dns_rules(tunnel, dns_servers, TransportProtocol::Udp)?;
                self.add_allow_dns_rules(tunnel, dns_servers, TransportProtocol::Tcp)?;
                // Important to block DNS *before* we allow the tunnel and allow LAN. So DNS
                // can't leak to the wrong IPs in the tunnel or on the LAN.
                self.add_drop_dns_rule();
                self.add_allow_tunnel_rules(&tunnel.interface)?;
                if *allow_lan {
                    self.add_block_cve_2019_14899(tunnel);
                }
                *allow_lan
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
            } => {
                if let Some(endpoint) = allowed_endpoint {
                    self.add_allow_endpoint_rules(&endpoint.endpoint);
                }

                // Important to drop DNS before allowing LAN (to stop DNS leaking to the LAN)
                self.add_drop_dns_rule();
                *allow_lan
            }
        };

        if allow_lan {
            self.add_allow_lan_rules();
        }

        // Reject any remaining outgoing traffic
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut reject_rule = Rule::new(chain);
            add_verdict(
                &mut reject_rule,
                &Verdict::Reject(RejectionType::Icmp(IcmpCode::PortUnreach)),
            );
            self.batch.add(&reject_rule, nftnl::MsgType::Add);
        }

        Ok(())
    }

    fn add_allow_tunnel_endpoint_rules(&mut self, endpoint: &Endpoint, fwmark: u32) {
        let mut prerouting_rule = Rule::new(&self.prerouting_chain);
        check_endpoint(&mut prerouting_rule, End::Src, endpoint);
        prerouting_rule.add_expr(&nft_expr!(immediate data fwmark));
        prerouting_rule.add_expr(&nft_expr!(meta mark set));

        if *ADD_COUNTERS {
            prerouting_rule.add_expr(&nft_expr!(counter));
        }

        self.batch.add(&prerouting_rule, nftnl::MsgType::Add);

        let mut in_rule = Rule::new(&self.in_chain);
        check_endpoint(&mut in_rule, End::Src, endpoint);

        in_rule.add_expr(&nft_expr!(ct state));
        let allowed_states = nftnl::expr::ct::States::ESTABLISHED.bits();
        in_rule.add_expr(&nft_expr!(bitwise mask allowed_states, xor 0u32));
        in_rule.add_expr(&nft_expr!(cmp != 0u32));
        add_verdict(&mut in_rule, &Verdict::Accept);

        self.batch.add(&in_rule, nftnl::MsgType::Add);

        let mut out_rule = Rule::new(&self.out_chain);
        check_endpoint(&mut out_rule, End::Dst, endpoint);
        out_rule.add_expr(&nft_expr!(meta mark));
        out_rule.add_expr(&nft_expr!(cmp == fwmark));
        add_verdict(&mut out_rule, &Verdict::Accept);

        self.batch.add(&out_rule, nftnl::MsgType::Add);
    }

    /// Adds firewall rules allow traffic to flow to the API. Allows the app to reach the API in
    /// blocked states.
    fn add_allow_endpoint_rules(&mut self, endpoint: &Endpoint) {
        let mut in_rule = Rule::new(&self.in_chain);
        check_endpoint(&mut in_rule, End::Src, endpoint);
        let allowed_states = nftnl::expr::ct::States::ESTABLISHED.bits();
        in_rule.add_expr(&nft_expr!(ct state));
        in_rule.add_expr(&nft_expr!(bitwise mask allowed_states, xor 0u32));
        in_rule.add_expr(&nft_expr!(cmp != 0u32));
        in_rule.add_expr(&nft_expr!(meta skuid));
        in_rule.add_expr(&nft_expr!(cmp == super::ROOT_UID));

        add_verdict(&mut in_rule, &Verdict::Accept);

        self.batch.add(&in_rule, nftnl::MsgType::Add);

        let mut out_rule = Rule::new(&self.out_chain);
        check_endpoint(&mut out_rule, End::Dst, endpoint);
        out_rule.add_expr(&nft_expr!(meta skuid));
        out_rule.add_expr(&nft_expr!(cmp == super::ROOT_UID));
        add_verdict(&mut out_rule, &Verdict::Accept);

        self.batch.add(&out_rule, nftnl::MsgType::Add);
    }

    fn add_allow_dns_rules(
        &mut self,
        tunnel: &tunnel::TunnelMetadata,
        dns_servers: &[IpAddr],
        protocol: TransportProtocol,
    ) -> Result<()> {
        let (local_resolvers, remote_resolvers): (Vec<IpAddr>, Vec<IpAddr>) = dns_servers
            .iter()
            .partition(|server| is_local_dns_address(tunnel, server));

        for resolver in &local_resolvers {
            self.add_allow_local_dns_rule(&tunnel.interface, protocol, *resolver)?;
        }

        for resolver in &remote_resolvers {
            self.add_allow_tunnel_dns_rule(&tunnel.interface, protocol, *resolver)?;
        }

        Ok(())
    }

    fn add_allow_tunnel_dns_rule(
        &mut self,
        interface: &str,
        protocol: TransportProtocol,
        host: IpAddr,
    ) -> Result<()> {
        for chain in &[&self.out_chain, &self.forward_chain] {
            let allow_rule = allow_tunnel_dns_rule(chain, interface, protocol, host)?;
            self.batch.add(&allow_rule, nftnl::MsgType::Add);
        }
        Ok(())
    }

    fn add_allow_local_dns_rule(
        &mut self,
        tunnel_interface: &str,
        protocol: TransportProtocol,
        host: IpAddr,
    ) -> Result<()> {
        let chains = [
            (&self.out_chain, Direction::Out),
            (&self.forward_chain, Direction::Out),
            (&self.in_chain, Direction::In),
            (&self.forward_chain, Direction::In),
        ];

        for (chain, direction) in &chains {
            let mut allow_rule = Rule::new(chain);
            let addr = match (host, direction) {
                (IpAddr::V4(_), Direction::Out) => nft_expr!(payload ipv4 daddr),
                (IpAddr::V6(_), Direction::Out) => nft_expr!(payload ipv6 daddr),
                (IpAddr::V4(_), Direction::In) => nft_expr!(payload ipv4 saddr),
                (IpAddr::V6(_), Direction::In) => nft_expr!(payload ipv6 saddr),
            };

            let port_dir = match direction {
                Direction::In => End::Src,
                Direction::Out => End::Dst,
            };

            check_not_iface(&mut allow_rule, *direction, tunnel_interface)?;
            check_port(&mut allow_rule, protocol, port_dir, 53);
            check_l3proto(&mut allow_rule, host);

            allow_rule.add_expr(&addr);
            allow_rule.add_expr(&nft_expr!(cmp == host));
            add_verdict(&mut allow_rule, &Verdict::Accept);

            self.batch.add(&allow_rule, nftnl::MsgType::Add);
        }

        Ok(())
    }

    /// Blocks all outgoing DNS (port 53) on both TCP and UDP
    fn add_drop_dns_rule(&mut self) {
        for chain in &[&self.out_chain, &self.forward_chain] {
            let mut block_udp_rule = Rule::new(chain);
            check_port(&mut block_udp_rule, TransportProtocol::Udp, End::Dst, 53);
            add_verdict(
                &mut block_udp_rule,
                &Verdict::Reject(RejectionType::Icmp(IcmpCode::PortUnreach)),
            );
            self.batch.add(&block_udp_rule, nftnl::MsgType::Add);

            let mut block_tcp_rule = Rule::new(chain);
            check_port(&mut block_tcp_rule, TransportProtocol::Tcp, End::Dst, 53);
            add_verdict(&mut block_tcp_rule, &Verdict::Reject(RejectionType::TcpRst));
            self.batch.add(&block_tcp_rule, nftnl::MsgType::Add);
        }
    }

    fn add_allow_in_tunnel_endpoint_rules(
        &mut self,
        tunnel_interface: &str,
        endpoint: &Endpoint,
    ) -> Result<()> {
        for (chain, dir, end) in [
            (&self.out_chain, Direction::Out, End::Dst),
            (&self.in_chain, Direction::In, End::Src),
        ] {
            let mut rule = Rule::new(chain);
            check_iface(&mut rule, dir, tunnel_interface)?;
            check_ip(&mut rule, end, endpoint.address.ip());
            check_port(&mut rule, endpoint.protocol, end, endpoint.address.port());
            add_verdict(&mut rule, &Verdict::Accept);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
        Ok(())
    }

    fn add_allow_tunnel_rules(&mut self, tunnel_interface: &str) -> Result<()> {
        self.batch.add(
            &allow_interface_rule(&self.out_chain, Direction::Out, tunnel_interface)?,
            nftnl::MsgType::Add,
        );
        self.batch.add(
            &allow_interface_rule(&self.forward_chain, Direction::Out, tunnel_interface)?,
            nftnl::MsgType::Add,
        );
        self.batch.add(
            &allow_interface_rule(&self.in_chain, Direction::In, tunnel_interface)?,
            nftnl::MsgType::Add,
        );

        let mut interface_rule = Rule::new(&self.forward_chain);
        check_iface(&mut interface_rule, Direction::In, tunnel_interface)?;
        interface_rule.add_expr(&nft_expr!(ct state));
        let allowed_states = nftnl::expr::ct::States::ESTABLISHED.bits();
        interface_rule.add_expr(&nft_expr!(bitwise mask allowed_states, xor 0u32));
        interface_rule.add_expr(&nft_expr!(cmp != 0u32));
        add_verdict(&mut interface_rule, &Verdict::Accept);
        self.batch.add(&interface_rule, nftnl::MsgType::Add);

        Ok(())
    }

    /// Adds rules for stopping [CVE-2019-14899](https://seclists.org/oss-sec/2019/q4/122).
    /// An attacker on the same local network as the VPN connected device could figure out
    /// the tunnel IP the device used if the device was set to not filter reverse path (rp_filter.)
    /// These rules stops all packets coming in to the tunnel IP. As such, these rules must come
    /// after the rule allowing the tunnel, otherwise even the tunnel can't talk to that IP.
    fn add_block_cve_2019_14899(&mut self, tunnel: &tunnel::TunnelMetadata) {
        for tunnel_ip in &tunnel.ips {
            let mut rule = Rule::new(&self.in_chain);
            check_ip(&mut rule, End::Dst, *tunnel_ip);
            add_verdict(&mut rule, &Verdict::Drop);
            self.batch.add(&rule, nftnl::MsgType::Add);
        }
    }

    fn add_allow_lan_rules(&mut self) {
        // Output and forward chains
        for chain in &[&self.out_chain, &self.forward_chain] {
            // LAN -> LAN
            for net in &*super::ALLOWED_LAN_NETS {
                let mut out_rule = Rule::new(chain);
                check_net(&mut out_rule, End::Dst, *net);
                add_verdict(&mut out_rule, &Verdict::Accept);
                self.batch.add(&out_rule, nftnl::MsgType::Add);
            }

            // LAN -> Multicast
            for net in &*super::ALLOWED_LAN_MULTICAST_NETS {
                let mut rule = Rule::new(chain);
                check_net(&mut rule, End::Dst, *net);
                add_verdict(&mut rule, &Verdict::Accept);
                self.batch.add(&rule, nftnl::MsgType::Add);
            }
        }

        // Input chain
        // LAN -> LAN
        for net in &*super::ALLOWED_LAN_NETS {
            let mut in_rule = Rule::new(&self.in_chain);
            check_net(&mut in_rule, End::Src, *net);
            add_verdict(&mut in_rule, &Verdict::Accept);
            self.batch.add(&in_rule, nftnl::MsgType::Add);
        }
        self.add_dhcp_server_rules();
    }

    fn add_dhcp_server_rules(&mut self) {
        use TransportProtocol::Udp;
        // Outgoing DHCPv4 response
        {
            let mut out_v4 = Rule::new(&self.out_chain);
            check_port(&mut out_v4, Udp, End::Src, super::DHCPV4_SERVER_PORT);
            check_port(&mut out_v4, Udp, End::Dst, super::DHCPV4_CLIENT_PORT);
            add_verdict(&mut out_v4, &Verdict::Accept);
            self.batch.add(&out_v4, nftnl::MsgType::Add);
        }
        // Incoming DHCPv4 request
        {
            let mut in_v4 = Rule::new(&self.in_chain);
            check_port(&mut in_v4, Udp, End::Src, super::DHCPV4_CLIENT_PORT);
            check_endpoint(
                &mut in_v4,
                End::Dst,
                &Endpoint::new(Ipv4Addr::BROADCAST, super::DHCPV4_SERVER_PORT, Udp),
            );
            add_verdict(&mut in_v4, &Verdict::Accept);
            self.batch.add(&in_v4, nftnl::MsgType::Add);
        }
    }
}

fn is_local_dns_address(tunnel: &tunnel::TunnelMetadata, server: &IpAddr) -> bool {
    super::is_local_address(server)
        && server != &tunnel.ipv4_gateway
        && Some(server) != tunnel.ipv6_gateway.map(IpAddr::from).as_ref()
}

fn allow_tunnel_dns_rule<'a>(
    chain: &'a Chain<'_>,
    iface: &str,
    protocol: TransportProtocol,
    host: IpAddr,
) -> Result<Rule<'a>> {
    let mut rule = Rule::new(chain);
    check_iface(&mut rule, Direction::Out, iface)?;
    check_port(&mut rule, protocol, End::Dst, 53);

    let daddr = match host {
        IpAddr::V4(_) => nft_expr!(payload ipv4 daddr),
        IpAddr::V6(_) => nft_expr!(payload ipv6 daddr),
    };
    if chain.get_table().get_family() == ProtoFamily::Inet {
        check_l3proto(&mut rule, host);
    }

    rule.add_expr(&daddr);
    rule.add_expr(&nft_expr!(cmp == host));
    add_verdict(&mut rule, &Verdict::Accept);

    Ok(rule)
}

fn allow_interface_rule<'a>(
    chain: &'a Chain<'_>,
    direction: Direction,
    iface: &str,
) -> Result<Rule<'a>> {
    let mut rule = Rule::new(chain);
    check_iface(&mut rule, direction, iface)?;
    add_verdict(&mut rule, &Verdict::Accept);

    Ok(rule)
}

fn check_iface(rule: &mut Rule<'_>, direction: Direction, iface: &str) -> Result<()> {
    let iface_index = crate::linux::iface_index(iface)
        .map_err(|e| Error::LookupIfaceIndexError(iface.to_owned(), e))?;
    rule.add_expr(&match direction {
        Direction::In => nft_expr!(meta iif),
        Direction::Out => nft_expr!(meta oif),
    });
    rule.add_expr(&nft_expr!(cmp == iface_index));
    Ok(())
}

fn check_not_iface(rule: &mut Rule<'_>, direction: Direction, iface: &str) -> Result<()> {
    let iface_index = crate::linux::iface_index(iface)
        .map_err(|e| Error::LookupIfaceIndexError(iface.to_owned(), e))?;
    rule.add_expr(&match direction {
        Direction::In => nft_expr!(meta iif),
        Direction::Out => nft_expr!(meta oif),
    });
    rule.add_expr(&nft_expr!(cmp != iface_index));
    Ok(())
}

fn check_net(rule: &mut Rule<'_>, end: End, net: impl Into<IpNetwork>) {
    let net = net.into();
    // Must check network layer protocol before loading network layer payload
    check_l3proto(rule, net.ip());

    rule.add_expr(&match (net, end) {
        (IpNetwork::V4(_), End::Src) => nft_expr!(payload ipv4 saddr),
        (IpNetwork::V4(_), End::Dst) => nft_expr!(payload ipv4 daddr),
        (IpNetwork::V6(_), End::Src) => nft_expr!(payload ipv6 saddr),
        (IpNetwork::V6(_), End::Dst) => nft_expr!(payload ipv6 daddr),
    });
    match net {
        IpNetwork::V4(_) => rule.add_expr(&nft_expr!(bitwise mask net.mask(), xor 0u32)),
        IpNetwork::V6(_) => rule.add_expr(&nft_expr!(bitwise mask net.mask(), xor &[0u16; 8][..])),
    };
    rule.add_expr(&nft_expr!(cmp == net.ip()));
}

fn check_icmpv6(rule: &mut Rule<'_>, r#type: u8, code: u8) {
    rule.add_expr(&nft_expr!(meta l4proto));
    rule.add_expr(&nft_expr!(cmp == libc::IPPROTO_ICMPV6 as u8));

    rule.add_expr(&Payload::Transport(
        nftnl::expr::TransportHeaderField::Icmpv6(nftnl::expr::Icmpv6HeaderField::Type),
    ));
    rule.add_expr(&nft_expr!(cmp == r#type));
    rule.add_expr(&nftnl::expr::Payload::Transport(
        nftnl::expr::TransportHeaderField::Icmpv6(nftnl::expr::Icmpv6HeaderField::Code),
    ));
    rule.add_expr(&nft_expr!(cmp == code));
}

fn check_endpoint(rule: &mut Rule<'_>, end: End, endpoint: &Endpoint) {
    check_ip(rule, end, endpoint.address.ip());
    check_port(rule, endpoint.protocol, end, endpoint.address.port());
}

fn check_ip(rule: &mut Rule<'_>, end: End, ip: impl Into<IpAddr>) {
    let ip = ip.into();
    // Must check network layer protocol before loading network layer payload
    check_l3proto(rule, ip);

    rule.add_expr(&match (ip, end) {
        (IpAddr::V4(..), End::Src) => nft_expr!(payload ipv4 saddr),
        (IpAddr::V4(..), End::Dst) => nft_expr!(payload ipv4 daddr),
        (IpAddr::V6(..), End::Src) => nft_expr!(payload ipv6 saddr),
        (IpAddr::V6(..), End::Dst) => nft_expr!(payload ipv6 daddr),
    });
    match ip {
        IpAddr::V4(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
        IpAddr::V6(addr) => rule.add_expr(&nft_expr!(cmp == addr)),
    }
}

fn check_port(rule: &mut Rule<'_>, protocol: TransportProtocol, end: End, port: u16) {
    // Must check transport layer protocol before loading transport layer payload
    check_l4proto(rule, protocol);

    rule.add_expr(&match (protocol, end) {
        (TransportProtocol::Udp, End::Src) => nft_expr!(payload udp sport),
        (TransportProtocol::Udp, End::Dst) => nft_expr!(payload udp dport),
        (TransportProtocol::Tcp, End::Src) => nft_expr!(payload tcp sport),
        (TransportProtocol::Tcp, End::Dst) => nft_expr!(payload tcp dport),
    });
    rule.add_expr(&nft_expr!(cmp == port.to_be()));
}

fn check_l3proto(rule: &mut Rule<'_>, ip: IpAddr) {
    rule.add_expr(&nft_expr!(meta nfproto));
    rule.add_expr(&nft_expr!(cmp == l3proto(ip)));
}

fn l3proto(addr: IpAddr) -> u8 {
    match addr {
        IpAddr::V4(_) => libc::NFPROTO_IPV4 as u8,
        IpAddr::V6(_) => libc::NFPROTO_IPV6 as u8,
    }
}

fn check_l4proto(rule: &mut Rule<'_>, protocol: TransportProtocol) {
    rule.add_expr(&nft_expr!(meta l4proto));
    rule.add_expr(&nft_expr!(cmp == l4proto(protocol)));
}

fn l4proto(protocol: TransportProtocol) -> u8 {
    match protocol {
        TransportProtocol::Udp => libc::IPPROTO_UDP as u8,
        TransportProtocol::Tcp => libc::IPPROTO_TCP as u8,
    }
}

fn add_verdict(rule: &mut Rule<'_>, verdict: &expr::Verdict) {
    if *ADD_COUNTERS {
        rule.add_expr(&nft_expr!(counter));
    }
    rule.add_expr(verdict);
}

fn set_src_valid_mark_sysctl() -> io::Result<()> {
    fs::write(PROC_SYS_NET_IPV4_CONF_SRC_VALID_MARK, b"1")
}
