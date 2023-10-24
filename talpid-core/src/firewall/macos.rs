use super::{FirewallArguments, FirewallPolicy};
use ipnetwork::IpNetwork;
use pfctl::{DropAction, FilterRuleAction, Uid};
use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};
use subslice::SubsliceExt;
use talpid_types::net::{self, AllowedEndpoint, AllowedTunnelTraffic};

pub use pfctl::Error;

type Result<T> = std::result::Result<T, Error>;

/// TODO(linus): This crate is not supposed to be Mullvad-aware. So at some point this should be
/// replaced by allowing the anchor name to be configured from the public API of this crate.
const ANCHOR_NAME: &str = "mullvad";

pub struct Firewall {
    pf: pfctl::PfCtl,
    pf_was_enabled: Option<bool>,
    rule_logging: RuleLogging,
}

impl Firewall {
    pub fn from_args(_args: FirewallArguments) -> Result<Self> {
        Self::new()
    }

    pub fn new() -> Result<Self> {
        // Allows controlling whether firewall rules should log to pflog0. Useful for debugging the
        // rules.
        let firewall_debugging = env::var("TALPID_FIREWALL_DEBUG");
        let rule_logging = match firewall_debugging.as_ref().map(String::as_str) {
            Ok("pass") => RuleLogging::Pass,
            Ok("drop") => RuleLogging::Drop,
            Ok("all") => RuleLogging::All,
            Ok(_) | Err(_) => RuleLogging::None,
        };
        log::trace!("Firewall debug log policy: {:?}", rule_logging);

        Ok(Firewall {
            pf: pfctl::PfCtl::new()?,
            pf_was_enabled: None,
            rule_logging,
        })
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<()> {
        self.enable()?;
        self.add_anchor()?;
        self.set_rules(policy)
    }

    pub fn reset_policy(&mut self) -> Result<()> {
        // Implemented this way to not early return on an error.
        // We always want all three methods to run, and then return
        // the first error it encountered, if any.
        self.remove_rules()
            .and(self.remove_anchor())
            .and(self.restore_state())
    }

    fn set_rules(&mut self, policy: FirewallPolicy) -> Result<()> {
        let mut new_filter_rules = vec![];

        new_filter_rules.append(&mut self.get_allow_loopback_rules()?);
        new_filter_rules.append(&mut self.get_allow_dhcp_client_rules()?);
        new_filter_rules.append(&mut self.get_allow_ndp_rules()?);
        new_filter_rules.append(&mut self.get_policy_specific_rules(&policy)?);

        let return_out_rule = self
            .create_rule_builder(FilterRuleAction::Drop(DropAction::Return))
            .direction(pfctl::Direction::Out)
            .quick(true)
            .build()?;
        new_filter_rules.push(return_out_rule);

        let drop_all_rule = self
            .create_rule_builder(FilterRuleAction::Drop(DropAction::Drop))
            .quick(true)
            .build()?;
        new_filter_rules.push(drop_all_rule);

        let mut anchor_change = pfctl::AnchorChange::new();
        anchor_change.set_filter_rules(new_filter_rules);
        anchor_change.set_redirect_rules(self.get_dns_redirect_rules(&policy)?);
        self.pf.set_rules(ANCHOR_NAME, anchor_change)
    }

    fn get_dns_redirect_rules(
        &mut self,
        policy: &FirewallPolicy,
    ) -> Result<Vec<pfctl::RedirectRule>> {
        let redirect_rules = match policy {
            FirewallPolicy::Blocked {
                dns_redirect_port, ..
            } => {
                vec![pfctl::RedirectRuleBuilder::default()
                    .action(pfctl::RedirectRuleAction::Redirect)
                    .interface("lo0")
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Port::from(53))
                    .redirect_to(pfctl::Port::from(*dns_redirect_port))
                    .build()?]
            }
            _ => vec![],
        };
        Ok(redirect_rules)
    }

    fn get_policy_specific_rules(
        &mut self,
        policy: &FirewallPolicy,
    ) -> Result<Vec<pfctl::FilterRule>> {
        match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
            } => {
                let mut rules = vec![self.get_allow_relay_rule(*peer_endpoint)?];
                rules.push(self.get_allowed_endpoint_rule(allowed_endpoint.clone())?);

                // Important to block DNS after allow relay rule (so the relay can operate
                // over port 53) but before allow LAN (so DNS does not leak to the LAN)
                rules.append(&mut self.get_block_dns_rules()?);

                if let Some(tunnel) = tunnel {
                    rules.extend(
                        self.get_allow_tunnel_rules(&tunnel.interface, allowed_tunnel_traffic)?,
                    );
                }

                if *allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }
                Ok(rules)
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
                dns_servers,
            } => {
                let mut rules = vec![];

                for server in dns_servers.iter() {
                    rules.append(&mut self.get_allow_dns_rules_when_connected(tunnel, *server)?);
                }

                rules.push(self.get_allow_relay_rule(*peer_endpoint)?);

                // Important to block DNS *before* we allow the tunnel and allow LAN. So DNS
                // can't leak to the wrong IPs in the tunnel or on the LAN.
                rules.append(&mut self.get_block_dns_rules()?);

                rules.extend(self.get_allow_tunnel_rules(
                    tunnel.interface.as_str(),
                    &AllowedTunnelTraffic::All,
                )?);

                if *allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }

                Ok(rules)
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
                ..
            } => {
                let mut rules = Vec::new();
                if let Some(allowed_endpoint) = allowed_endpoint {
                    rules.push(self.get_allowed_endpoint_rule(allowed_endpoint.clone())?);
                }

                if *allow_lan {
                    // Important to block DNS before allow LAN (so DNS does not leak to the LAN)
                    rules.append(&mut self.get_block_dns_rules()?);
                    rules.append(&mut self.get_allow_lan_rules()?);
                }

                Ok(rules)
            }
        }
    }

    fn get_allow_dns_rules_when_connected(
        &self,
        tunnel: &crate::tunnel::TunnelMetadata,
        server: IpAddr,
    ) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = Vec::with_capacity(4);

        let is_local = super::is_local_address(&server)
            && server != tunnel.ipv4_gateway
            && !tunnel
                .ipv6_gateway
                .map(|ref gateway| &server == gateway)
                .unwrap_or(false);

        if is_local {
            // Block requests on the tunnel interface
            let block_tunnel_tcp = self
                .create_rule_builder(FilterRuleAction::Drop(DropAction::Return))
                .direction(pfctl::Direction::Out)
                .quick(true)
                .interface(&tunnel.interface)
                .proto(pfctl::Proto::Tcp)
                .keep_state(pfctl::StatePolicy::None)
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(block_tunnel_tcp);
            let block_tunnel_udp = self
                .create_rule_builder(FilterRuleAction::Drop(DropAction::Return))
                .direction(pfctl::Direction::Out)
                .quick(true)
                .interface(&tunnel.interface)
                .proto(pfctl::Proto::Udp)
                .keep_state(pfctl::StatePolicy::None)
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(block_tunnel_udp);

            // Allow requests on other interfaces
            let allow_nontunnel_tcp = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::Out)
                .quick(true)
                .proto(pfctl::Proto::Tcp)
                .keep_state(pfctl::StatePolicy::Keep)
                .tcp_flags(Self::get_tcp_flags())
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(allow_nontunnel_tcp);
            let allow_nontunnel_udp = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::Out)
                .quick(true)
                .proto(pfctl::Proto::Udp)
                .keep_state(pfctl::StatePolicy::Keep)
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(allow_nontunnel_udp);
        } else {
            // Allow outgoing requests on the tunnel interface only
            let allow_tunnel_tcp = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::Out)
                .quick(true)
                .interface(&tunnel.interface)
                .proto(pfctl::Proto::Tcp)
                .keep_state(pfctl::StatePolicy::Keep)
                .tcp_flags(Self::get_tcp_flags())
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(allow_tunnel_tcp);
            let allow_tunnel_udp = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::Out)
                .quick(true)
                .interface(&tunnel.interface)
                .proto(pfctl::Proto::Udp)
                .to(pfctl::Endpoint::new(server, 53))
                .build()?;
            rules.push(allow_tunnel_udp);
        };

        Ok(rules)
    }

    fn get_allow_relay_rule(&self, relay_endpoint: net::Endpoint) -> Result<pfctl::FilterRule> {
        let pfctl_proto = as_pfctl_proto(relay_endpoint.protocol);

        self.create_rule_builder(FilterRuleAction::Pass)
            .direction(pfctl::Direction::Out)
            .to(relay_endpoint.address)
            .proto(pfctl_proto)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .user(Uid::from(super::ROOT_UID))
            .quick(true)
            .build()
    }

    /// Produces a rule that allows traffic to flow to the API. Allows the app (or other apps if configured)
    /// to reach the API in blocked states.
    fn get_allowed_endpoint_rule(
        &self,
        allowed_endpoint: AllowedEndpoint,
    ) -> Result<pfctl::FilterRule> {
        let pfctl_proto = as_pfctl_proto(allowed_endpoint.endpoint.protocol);

        let mut rule = self.create_rule_builder(FilterRuleAction::Pass);
        rule.direction(pfctl::Direction::Out)
            .to(allowed_endpoint.endpoint.address)
            .proto(pfctl_proto)
            .keep_state(pfctl::StatePolicy::Keep)
            .quick(true);

        if !allowed_endpoint.clients.allow_all() {
            rule.user(Uid::from(super::ROOT_UID)).build()?;
        }

        rule.build()
    }

    fn get_block_dns_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let block_tcp_dns_rule = self
            .create_rule_builder(FilterRuleAction::Drop(DropAction::Return))
            .direction(pfctl::Direction::Out)
            .quick(true)
            .proto(pfctl::Proto::Tcp)
            .to(pfctl::Port::from(53))
            .build()?;
        let block_udp_dns_rule = self
            .create_rule_builder(FilterRuleAction::Drop(DropAction::Return))
            .direction(pfctl::Direction::Out)
            .quick(true)
            .proto(pfctl::Proto::Udp)
            .to(pfctl::Port::from(53))
            .build()?;

        Ok(vec![block_tcp_dns_rule, block_udp_dns_rule])
    }

    fn base_rule(
        &self,
        action: FilterRuleAction,
        tunnel_interface: &str,
    ) -> pfctl::FilterRuleBuilder {
        let mut rule_builder = self.create_rule_builder(action);
        rule_builder
            .quick(true)
            .interface(tunnel_interface)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags());
        rule_builder
    }

    fn get_allow_tunnel_rules(
        &self,
        tunnel_interface: &str,
        allowed_traffic: &AllowedTunnelTraffic,
    ) -> Result<Vec<pfctl::FilterRule>> {
        Ok(match allowed_traffic {
            AllowedTunnelTraffic::One(endpoint) => {
                let mut base_rule = &mut self.base_rule(FilterRuleAction::Pass, tunnel_interface);
                let pfctl_proto = as_pfctl_proto(endpoint.protocol);
                base_rule = base_rule.to(endpoint.address).proto(pfctl_proto);
                vec![base_rule.build()?]
            }
            AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
                let mut rules = Vec::with_capacity(2);

                let mut base_rule = &mut self.base_rule(FilterRuleAction::Pass, tunnel_interface);
                let pfctl_proto = as_pfctl_proto(endpoint1.protocol);
                base_rule = base_rule.to(endpoint1.address).proto(pfctl_proto);
                rules.push(base_rule.build()?);

                let mut base_rule = &mut self.base_rule(FilterRuleAction::Pass, tunnel_interface);
                let pfctl_proto = as_pfctl_proto(endpoint2.protocol);
                base_rule = base_rule.to(endpoint2.address).proto(pfctl_proto);
                rules.push(base_rule.build()?);

                rules
            }
            AllowedTunnelTraffic::All => {
                let base_rule = &mut self.base_rule(FilterRuleAction::Pass, tunnel_interface);
                vec![base_rule.build()?]
            }
            AllowedTunnelTraffic::None => {
                vec![]
            }
        })
    }

    fn get_allow_loopback_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let lo0_rule = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .interface("lo0")
            .keep_state(pfctl::StatePolicy::Keep)
            .build()?;
        Ok(vec![lo0_rule])
    }

    fn get_allow_lan_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = vec![];
        for net in &*super::ALLOWED_LAN_NETS {
            let mut rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
            rule_builder.quick(true);
            let allow_out = rule_builder
                .direction(pfctl::Direction::Out)
                .from(pfctl::Ip::Any)
                .to(pfctl::Ip::from(*net))
                .build()?;
            let allow_in = rule_builder
                .direction(pfctl::Direction::In)
                .from(pfctl::Ip::from(*net))
                .to(pfctl::Ip::Any)
                .build()?;
            rules.push(allow_out);
            rules.push(allow_in);
        }
        for multicast_net in &*super::ALLOWED_LAN_MULTICAST_NETS {
            let allow_multicast_out = self
                .create_rule_builder(FilterRuleAction::Pass)
                .quick(true)
                .direction(pfctl::Direction::Out)
                .to(pfctl::Ip::from(*multicast_net))
                .build()?;
            rules.push(allow_multicast_out);
        }

        let dhcpv4_out = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .direction(pfctl::Direction::Out)
            .af(pfctl::AddrFamily::Ipv4)
            .proto(pfctl::Proto::Udp)
            .from(pfctl::Port::from(super::DHCPV4_SERVER_PORT))
            .to(pfctl::Port::from(super::DHCPV4_CLIENT_PORT))
            .build()?;
        let dhcpv4_in = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .direction(pfctl::Direction::In)
            .proto(pfctl::Proto::Udp)
            .from(pfctl::Port::from(super::DHCPV4_CLIENT_PORT))
            .to(pfctl::Endpoint::new(
                Ipv4Addr::BROADCAST,
                pfctl::Port::from(super::DHCPV4_SERVER_PORT),
            ))
            .build()?;
        rules.push(dhcpv4_out);
        rules.push(dhcpv4_in);

        Ok(rules)
    }

    fn get_allow_dhcp_client_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let mut dhcp_rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
        dhcp_rule_builder.quick(true).proto(pfctl::Proto::Udp);

        let mut rules = Vec::new();

        // DHCPv4
        dhcp_rule_builder.af(pfctl::AddrFamily::Ipv4);
        let allow_outgoing_dhcp_v4 = dhcp_rule_builder
            .direction(pfctl::Direction::Out)
            .from(pfctl::Port::from(super::DHCPV4_CLIENT_PORT))
            .to(pfctl::Endpoint::new(
                Ipv4Addr::BROADCAST,
                pfctl::Port::from(super::DHCPV4_SERVER_PORT),
            ))
            .build()?;
        let allow_incoming_dhcp_v4 = dhcp_rule_builder
            .direction(pfctl::Direction::In)
            .from(pfctl::Port::from(super::DHCPV4_SERVER_PORT))
            .to(pfctl::Port::from(super::DHCPV4_CLIENT_PORT))
            .build()?;
        rules.push(allow_outgoing_dhcp_v4);
        rules.push(allow_incoming_dhcp_v4);

        // DHCPv6
        dhcp_rule_builder.af(pfctl::AddrFamily::Ipv6);
        for dhcpv6_server in &*super::DHCPV6_SERVER_ADDRS {
            let allow_outgoing_dhcp_v6 = dhcp_rule_builder
                .direction(pfctl::Direction::Out)
                .from(pfctl::Endpoint::new(
                    IpNetwork::V6(*super::IPV6_LINK_LOCAL),
                    pfctl::Port::from(super::DHCPV6_CLIENT_PORT),
                ))
                .to(pfctl::Endpoint::new(
                    *dhcpv6_server,
                    pfctl::Port::from(super::DHCPV6_SERVER_PORT),
                ))
                .build()?;
            rules.push(allow_outgoing_dhcp_v6);
        }
        let allow_incoming_dhcp_v6 = dhcp_rule_builder
            .direction(pfctl::Direction::In)
            .from(pfctl::Endpoint::new(
                pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)),
                pfctl::Port::from(super::DHCPV6_SERVER_PORT),
            ))
            .to(pfctl::Endpoint::new(
                pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)),
                pfctl::Port::from(super::DHCPV6_CLIENT_PORT),
            ))
            .build()?;
        rules.push(allow_incoming_dhcp_v6);

        Ok(rules)
    }

    fn get_allow_ndp_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let mut ndp_rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
        ndp_rule_builder
            .quick(true)
            .af(pfctl::AddrFamily::Ipv6)
            .proto(pfctl::Proto::IcmpV6);

        Ok(vec![
            // Outgoing router solicitation to `ff02::2`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::Out)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::RouterSol))
                .to(*super::ROUTER_SOLICITATION_OUT_DST_ADDR)
                .build()?,
            // Incoming router advertisement from `fe80::/10`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::In)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::RouterAdv))
                .from(pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)))
                .build()?,
            // Incoming Redirect from `fe80::/10`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::In)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::Redir))
                .from(pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)))
                .build()?,
            // Outgoing neighbor solicitation to `ff02::1:ff00:0/104` and `fe80::/10`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::Out)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::NeighbrSol))
                .to(pfctl::Ip::from(IpNetwork::V6(
                    *super::SOLICITED_NODE_MULTICAST,
                )))
                .build()?,
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::Out)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::NeighbrSol))
                .to(pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)))
                .build()?,
            // Incoming neighbor solicitation from `fe80::/10`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::In)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::NeighbrSol))
                .from(pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)))
                .build()?,
            // Outgoing neighbor advertisement to fe80::/10`
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::Out)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::NeighbrAdv))
                .to(pfctl::Ip::from(IpNetwork::V6(*super::IPV6_LINK_LOCAL)))
                .build()?,
            // Incoming neighbor advertisement from anywhere
            ndp_rule_builder
                .clone()
                .direction(pfctl::Direction::In)
                .icmp_type(pfctl::IcmpType::Icmp6(pfctl::Icmp6Type::NeighbrAdv))
                .build()?,
        ])
    }

    fn create_rule_builder(&self, action: FilterRuleAction) -> pfctl::FilterRuleBuilder {
        let mut builder = pfctl::FilterRuleBuilder::default();
        builder.action(action);
        let rule_log = pfctl::RuleLog::IncludeMatchingState;
        let do_log = match action {
            FilterRuleAction::Pass => {
                matches!(self.rule_logging, RuleLogging::All | RuleLogging::Pass)
            }
            FilterRuleAction::Drop(..) => {
                matches!(self.rule_logging, RuleLogging::All | RuleLogging::Drop)
            }
        };
        if do_log {
            builder.log(rule_log);
        }
        builder
    }

    fn get_tcp_flags() -> pfctl::TcpFlags {
        pfctl::TcpFlags::new(
            &[pfctl::TcpFlag::Syn],
            &[pfctl::TcpFlag::Syn, pfctl::TcpFlag::Ack],
        )
    }

    fn remove_rules(&mut self) -> Result<()> {
        // remove_anchor() does not deactivate active rules
        self.pf
            .flush_rules(ANCHOR_NAME, pfctl::RulesetKind::Filter)?;
        Ok(())
    }

    fn enable(&mut self) -> Result<()> {
        if self.pf_was_enabled.is_none() {
            self.pf_was_enabled = Some(self.is_enabled());
        }
        self.pf.try_enable()
    }

    fn is_enabled(&self) -> bool {
        let cmd = duct::cmd!("/sbin/pfctl", "-s", "info")
            .stderr_null()
            .stdout_capture();
        const EXPECTED_OUTPUT: &[u8] = b"Status: Enabled";
        match cmd.run() {
            Ok(output) => output.stdout.as_slice().find(EXPECTED_OUTPUT).is_some(),
            Err(err) => {
                log::error!(
                    "Failed to execute pfctl, assuming pf is not enabled: {}",
                    err
                );
                false
            }
        }
    }

    fn restore_state(&mut self) -> Result<()> {
        match self.pf_was_enabled.take() {
            Some(true) => Ok(self.pf.try_enable()?),
            Some(false) => Ok(self.pf.try_disable()?),
            None => Ok(()),
        }
    }

    fn add_anchor(&mut self) -> Result<()> {
        self.pf
            .try_add_anchor(ANCHOR_NAME, pfctl::AnchorKind::Filter)?;
        self.pf
            .try_add_anchor(ANCHOR_NAME, pfctl::AnchorKind::Redirect)?;
        Ok(())
    }

    fn remove_anchor(&mut self) -> Result<()> {
        self.pf
            .try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Filter)?;
        self.pf
            .try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Redirect)?;
        Ok(())
    }
}

fn as_pfctl_proto(protocol: net::TransportProtocol) -> pfctl::Proto {
    match protocol {
        net::TransportProtocol::Udp => pfctl::Proto::Udp,
        net::TransportProtocol::Tcp => pfctl::Proto::Tcp,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum RuleLogging {
    None,
    Pass,
    Drop,
    All,
}
