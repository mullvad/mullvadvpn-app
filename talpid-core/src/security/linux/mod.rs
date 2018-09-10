extern crate mnl;

use error_chain::ChainedError;

use ipnetwork::IpNetwork;
use libc;
use nftnl::{
    self,
    expr::{self, Verdict},
    Batch, Chain, FinalizedBatch, ProtoFamily, Rule, Table,
};
use talpid_types::net::{Endpoint, TransportProtocol};
use tunnel;

use std::env;
use std::ffi::CString;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

use super::{NetworkSecurityT, SecurityPolicy};

mod dns;
use self::dns::DnsSettings;

error_chain! {
    errors {
        /// Unable to open netlink socket to netfilter
        NetlinkOpenError { description("Unable to open netlink socket to netfilter") }
        /// Unable to send netlink command to netfilter
        NetlinkSendError { description("Unable to send netlink command to netfilter") }
        /// Error while reading from netlink socket
        NetlinkRecvError { description("Error while reading from netlink socket") }
        /// Error while processing an incoming netlink message
        ProcessNetlinkError { description("Error while processing an incoming netlink message") }
        /// The name is not a valid Linux network interface name
        InvalidInterfaceName(name: String) {
            description("Invalid network interface name")
            display("Invalid network interface name: {}", name)
        }
    }
    links {
        DnsSettings(self::dns::Error, self::dns::ErrorKind) #[doc = "DNS error"];
        Nftnl(nftnl::Error, nftnl::ErrorKind) #[doc = "Error in nftnl"];
    }
}

lazy_static! {
    /// TODO(linus): This crate is not supposed to be Mullvad-aware. So at some point this should be
    /// replaced by allowing the table name to be configured from the public API of this crate.
    static ref TABLE_NAME: CString = CString::new("mullvad").unwrap();
    static ref IN_CHAIN_NAME: CString = CString::new("in").unwrap();
    static ref OUT_CHAIN_NAME: CString = CString::new("out").unwrap();

    /// Allows controlling whether firewall rules should have packet counters or not from an env
    /// variable. Useful for debugging the rules.
    static ref ADD_COUNTERS: bool = env::var("TALPID_NFTABLES_COUNTERS")
        .map(|v| v == "1")
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
pub struct NetworkSecurity {
    dns_settings: DnsSettings,
    table_name: CString,
}

impl NetworkSecurityT for NetworkSecurity {
    type Error = Error;

    fn new(_cache_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(NetworkSecurity {
            dns_settings: DnsSettings::new()?,
            table_name: TABLE_NAME.clone(),
        })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        if let SecurityPolicy::Connected { ref tunnel, .. } = policy {
            self.dns_settings
                .set_dns(&tunnel.interface, vec![tunnel.gateway.into()])?;
        }

        let table = Table::new(&self.table_name, ProtoFamily::Inet)?;
        let batch = PolicyBatch::new(&table)?.finalize(&policy)?;
        self.send_and_process(&batch)
    }

    fn reset_policy(&mut self) -> Result<()> {
        if let Err(error) = self.dns_settings.reset() {
            error!("Failed to reset DNS settings: {}", error.display_chain());
        }

        let table = Table::new(&self.table_name, ProtoFamily::Inet)?;
        let batch = {
            let mut batch = Batch::new()?;
            // Our batch will add and remove the table even though the goal is just to remove it.
            // This because only removing it throws a strange error if the table does not exist.
            batch.add(&table, nftnl::MsgType::Add)?;
            batch.add(&table, nftnl::MsgType::Del)?;
            batch.finalize()?
        };

        debug!("Removing table and chain from netfilter");
        self.send_and_process(&batch)
    }
}

impl NetworkSecurity {
    fn send_and_process(&self, batch: &FinalizedBatch) -> Result<()> {
        let socket =
            mnl::Socket::new(mnl::Bus::Netfilter).chain_err(|| ErrorKind::NetlinkOpenError)?;
        socket
            .send_all(batch)
            .chain_err(|| ErrorKind::NetlinkSendError)?;

        let portid = socket.portid();
        let mut buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];


        while let Some(message) = Self::socket_recv(&socket, &mut buffer[..])? {
            match mnl::cb_run(message, 2, portid).chain_err(|| ErrorKind::ProcessNetlinkError)? {
                mnl::CbResult::Stop => {
                    trace!("cb_run STOP");
                    break;
                }
                mnl::CbResult::Ok => trace!("cb_run OK"),
            }
        }

        Ok(())
    }

    fn socket_recv<'a>(socket: &mnl::Socket, buf: &'a mut [u8]) -> Result<Option<&'a [u8]>> {
        let ret = socket.recv(buf).chain_err(|| ErrorKind::NetlinkRecvError)?;
        trace!("Read {} bytes from netlink", ret);
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
}

impl<'a> PolicyBatch<'a> {
    /// Bootstrap a new nftnl message batch object and add the initial messages creating the
    /// table and chains.
    pub fn new(table: &'a Table) -> Result<Self> {
        let mut batch = Batch::new()?;
        let mut out_chain = Chain::new(&*OUT_CHAIN_NAME, table)?;
        let mut in_chain = Chain::new(&*IN_CHAIN_NAME, table)?;
        out_chain.set_hook(nftnl::Hook::Out, 0);
        in_chain.set_hook(nftnl::Hook::In, 0);
        out_chain.set_policy(nftnl::Policy::Drop);
        in_chain.set_policy(nftnl::Policy::Drop);

        batch.add(table, nftnl::MsgType::Add)?;
        batch.add(table, nftnl::MsgType::Del)?;
        batch.add(table, nftnl::MsgType::Add)?;
        batch.add(&out_chain, nftnl::MsgType::Add)?;
        batch.add(&in_chain, nftnl::MsgType::Add)?;

        Ok(PolicyBatch {
            batch,
            in_chain,
            out_chain,
        })
    }

    /// Finalize the nftnl message batch by adding every firewall rule needed to satisfy the given
    /// policy.
    pub fn finalize(mut self, policy: &SecurityPolicy) -> Result<FinalizedBatch> {
        self.add_loopback_rules()?;
        self.add_dhcp_rules()?;
        self.add_policy_specific_rules(policy)?;

        Ok(self.batch.finalize()?)
    }

    fn add_loopback_rules(&mut self) -> Result<()> {
        const LOOPBACK_IFACE_NAME: &str = "lo";
        self.batch.add(
            &allow_interface_rule(&self.out_chain, Direction::Out, LOOPBACK_IFACE_NAME)?,
            nftnl::MsgType::Add,
        )?;
        self.batch.add(
            &allow_interface_rule(&self.in_chain, Direction::In, LOOPBACK_IFACE_NAME)?,
            nftnl::MsgType::Add,
        )?;
        Ok(())
    }

    fn add_dhcp_rules(&mut self) -> Result<()> {
        self.batch.add(
            &allow_dhcp_rule(&self.out_chain, Direction::Out)?,
            nftnl::MsgType::Add,
        )?;
        self.batch.add(
            &allow_dhcp_rule(&self.in_chain, Direction::In)?,
            nftnl::MsgType::Add,
        )?;
        Ok(())
    }

    fn add_policy_specific_rules(&mut self, policy: &SecurityPolicy) -> Result<()> {
        let allow_lan = match policy {
            SecurityPolicy::Connecting {
                relay_endpoint,
                allow_lan,
            } => {
                self.add_allow_endpoint_rules(relay_endpoint)?;
                *allow_lan
            }
            SecurityPolicy::Connected {
                relay_endpoint,
                tunnel,
                allow_lan,
            } => {
                self.add_allow_endpoint_rules(relay_endpoint)?;
                self.add_dns_rule(tunnel, TransportProtocol::Udp)?;
                self.add_dns_rule(tunnel, TransportProtocol::Tcp)?;
                self.add_allow_tunnel_rules(tunnel)?;
                *allow_lan
            }
            SecurityPolicy::Blocked { allow_lan } => *allow_lan,
        };

        if allow_lan {
            self.add_allow_lan_rules()?;
        }
        Ok(())
    }

    fn add_allow_endpoint_rules(&mut self, endpoint: &Endpoint) -> Result<()> {
        let mut in_rule = Rule::new(&self.in_chain)?;
        check_endpoint(&mut in_rule, End::Src, endpoint)?;

        in_rule.add_expr(&nft_expr!(ct state))?;
        let allowed_states = nftnl::expr::ct::States::ESTABLISHED.bits();
        in_rule.add_expr(&nft_expr!(bitwise mask allowed_states, xor 0u32))?;
        in_rule.add_expr(&nft_expr!(cmp != 0u32))?;
        add_verdict(&mut in_rule, Verdict::Accept)?;

        self.batch.add(&in_rule, nftnl::MsgType::Add)?;


        let mut out_rule = Rule::new(&self.out_chain)?;
        check_endpoint(&mut out_rule, End::Dst, endpoint)?;
        add_verdict(&mut out_rule, Verdict::Accept)?;

        self.batch.add(&out_rule, nftnl::MsgType::Add)?;

        Ok(())
    }

    fn add_dns_rule(
        &mut self,
        tunnel: &tunnel::TunnelMetadata,
        protocol: TransportProtocol,
    ) -> Result<()> {
        let mut rule = Rule::new(&self.out_chain)?;

        check_iface(&mut rule, Direction::Out, &tunnel.interface[..])?;
        check_port(&mut rule, protocol, End::Dst, 53)?;
        check_l3proto(&mut rule, IpAddr::V4(tunnel.gateway))?;

        rule.add_expr(&nft_expr!(payload ipv4 daddr))?;
        rule.add_expr(&nft_expr!(cmp != tunnel.gateway))?;

        add_verdict(&mut rule, Verdict::Drop)?;

        self.batch.add(&rule, nftnl::MsgType::Add)?;
        Ok(())
    }

    fn add_allow_tunnel_rules(&mut self, tunnel: &tunnel::TunnelMetadata) -> Result<()> {
        self.batch.add(
            &allow_interface_rule(&self.out_chain, Direction::Out, &tunnel.interface[..])?,
            nftnl::MsgType::Add,
        )?;
        self.batch.add(
            &allow_interface_rule(&self.in_chain, Direction::In, &tunnel.interface[..])?,
            nftnl::MsgType::Add,
        )?;
        Ok(())
    }

    fn add_allow_lan_rules(&mut self) -> Result<()> {
        // LAN -> LAN
        for chain in &[&self.in_chain, &self.out_chain] {
            for net in &*super::PRIVATE_NETS {
                let mut rule = Rule::new(chain)?;
                check_net(&mut rule, End::Src, IpNetwork::V4(*net))?;
                check_net(&mut rule, End::Dst, IpNetwork::V4(*net))?;
                add_verdict(&mut rule, Verdict::Accept)?;

                self.batch.add(&rule, nftnl::MsgType::Add)?;
            }
        }
        // LAN -> multicast
        for net in &*super::PRIVATE_NETS {
            let mut rule = Rule::new(&self.out_chain)?;
            check_net(&mut rule, End::Src, IpNetwork::V4(*net))?;
            check_net(&mut rule, End::Dst, IpNetwork::V4(*super::MULTICAST_NET))?;
            add_verdict(&mut rule, Verdict::Accept)?;

            self.batch.add(&rule, nftnl::MsgType::Add)?;

            // LAN -> SSDP + WS-Discovery protocols
            let mut rule = Rule::new(&self.out_chain)?;
            check_net(&mut rule, End::Src, IpNetwork::V4(*net))?;
            check_ip(&mut rule, End::Dst, *super::SSDP_IP)?;
            add_verdict(&mut rule, Verdict::Accept)?;

            self.batch.add(&rule, nftnl::MsgType::Add)?;
        }
        Ok(())
    }
}

fn allow_dhcp_rule<'a>(chain: &'a Chain, direction: Direction) -> Result<Rule<'a>> {
    const SERVER_PORT: u16 = 67;
    const CLIENT_PORT: u16 = 68;
    let broadcast_addr = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));

    let mut rule = Rule::new(&chain)?;

    match direction {
        Direction::In => {
            check_port(&mut rule, TransportProtocol::Udp, End::Src, SERVER_PORT)?;
            check_port(&mut rule, TransportProtocol::Udp, End::Dst, CLIENT_PORT)?;
        }
        Direction::Out => {
            check_port(&mut rule, TransportProtocol::Udp, End::Src, CLIENT_PORT)?;
            check_port(&mut rule, TransportProtocol::Udp, End::Dst, SERVER_PORT)?;
            check_ip(&mut rule, End::Dst, broadcast_addr)?;
        }
    }

    add_verdict(&mut rule, Verdict::Accept)?;

    Ok(rule)
}

fn allow_interface_rule<'a>(
    chain: &'a Chain,
    direction: Direction,
    iface: &str,
) -> Result<Rule<'a>> {
    let mut rule = Rule::new(&chain)?;
    check_iface(&mut rule, direction, iface)?;
    add_verdict(&mut rule, Verdict::Accept)?;

    Ok(rule)
}

fn check_iface(rule: &mut Rule, direction: Direction, iface: &str) -> Result<()> {
    let iface_index = iface_index(iface)?;
    rule.add_expr(&match direction {
        Direction::In => nft_expr!(meta iif),
        Direction::Out => nft_expr!(meta oif),
    })?;
    rule.add_expr(&nft_expr!(cmp == iface_index))?;
    Ok(())
}

fn iface_index(name: &str) -> Result<libc::c_uint> {
    let c_name =
        CString::new(name).chain_err(|| ErrorKind::InvalidInterfaceName(name.to_owned()))?;
    let index = unsafe { libc::if_nametoindex(c_name.as_ptr()) };
    if index == 0 {
        let error = io::Error::last_os_error();
        Err(error).chain_err(|| ErrorKind::InvalidInterfaceName(name.to_owned()))
    } else {
        Ok(index)
    }
}

fn check_net(rule: &mut Rule, end: End, net: IpNetwork) -> Result<()> {
    // Must check network layer protocol before loading network layer payload
    check_l3proto(rule, net.ip())?;

    rule.add_expr(&match (net, end) {
        (IpNetwork::V4(_), End::Src) => nft_expr!(payload ipv4 saddr),
        (IpNetwork::V4(_), End::Dst) => nft_expr!(payload ipv4 daddr),
        (IpNetwork::V6(_), End::Src) => nft_expr!(payload ipv6 saddr),
        (IpNetwork::V6(_), End::Dst) => nft_expr!(payload ipv6 daddr),
    })?;
    rule.add_expr(&nft_expr!(bitwise mask net.mask(), xor 0))?;
    rule.add_expr(&nft_expr!(cmp == net.ip()))?;

    Ok(())
}

fn check_endpoint(rule: &mut Rule, end: End, endpoint: &Endpoint) -> Result<()> {
    check_ip(rule, end, endpoint.address.ip())?;
    check_port(rule, endpoint.protocol, end, endpoint.address.port())?;
    Ok(())
}


fn check_ip(rule: &mut Rule, end: End, ip: IpAddr) -> Result<()> {
    // Must check network layer protocol before loading network layer payload
    check_l3proto(rule, ip)?;

    rule.add_expr(&match (ip, end) {
        (IpAddr::V4(..), End::Src) => nft_expr!(payload ipv4 saddr),
        (IpAddr::V4(..), End::Dst) => nft_expr!(payload ipv4 daddr),
        (IpAddr::V6(..), End::Src) => nft_expr!(payload ipv6 saddr),
        (IpAddr::V6(..), End::Dst) => nft_expr!(payload ipv6 daddr),
    })?;
    match ip {
        IpAddr::V4(addr) => rule.add_expr(&nft_expr!(cmp == addr))?,
        IpAddr::V6(addr) => rule.add_expr(&nft_expr!(cmp == addr))?,
    }
    Ok(())
}

fn check_port(rule: &mut Rule, protocol: TransportProtocol, end: End, port: u16) -> Result<()> {
    // Must check transport layer protocol before loading transport layer payload
    check_l4proto(rule, protocol)?;

    rule.add_expr(&match (protocol, end) {
        (TransportProtocol::Udp, End::Src) => nft_expr!(payload udp sport),
        (TransportProtocol::Udp, End::Dst) => nft_expr!(payload udp dport),
        (TransportProtocol::Tcp, End::Src) => nft_expr!(payload tcp sport),
        (TransportProtocol::Tcp, End::Dst) => nft_expr!(payload tcp dport),
    })?;
    rule.add_expr(&nft_expr!(cmp == port.to_be()))?;
    Ok(())
}

fn check_l3proto(rule: &mut Rule, ip: IpAddr) -> Result<()> {
    rule.add_expr(&nft_expr!(meta nfproto))?;
    rule.add_expr(&nft_expr!(cmp == l3proto(ip)))?;
    Ok(())
}

fn l3proto(addr: IpAddr) -> u8 {
    match addr {
        IpAddr::V4(_) => libc::NFPROTO_IPV4 as u8,
        IpAddr::V6(_) => libc::NFPROTO_IPV6 as u8,
    }
}

fn check_l4proto(rule: &mut Rule, protocol: TransportProtocol) -> Result<()> {
    rule.add_expr(&nft_expr!(meta l4proto))?;
    rule.add_expr(&nft_expr!(cmp == l4proto(protocol)))?;
    Ok(())
}

fn l4proto(protocol: TransportProtocol) -> u8 {
    match protocol {
        TransportProtocol::Udp => libc::IPPROTO_UDP as u8,
        TransportProtocol::Tcp => libc::IPPROTO_TCP as u8,
    }
}

fn add_verdict(rule: &mut Rule, verdict: expr::Verdict) -> Result<()> {
    if *ADD_COUNTERS {
        rule.add_expr(&nft_expr!(counter))?;
    }
    Ok(rule.add_expr(&verdict)?)
}
