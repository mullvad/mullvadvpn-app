use super::{FirewallPolicy, FirewallT};
use pfctl::FilterRuleAction;
use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};
use talpid_types::net;

pub use pfctl::Error;

type Result<T> = ::std::result::Result<T, Error>;

/// TODO(linus): This crate is not supposed to be Mullvad-aware. So at some point this should be
/// replaced by allowing the anchor name to be configured from the public API of this crate.
const ANCHOR_NAME: &'static str = "mullvad";

/// The macOS firewall and DNS implementation.
pub struct Firewall {
    pf: pfctl::PfCtl,
    pf_was_enabled: Option<bool>,
    rule_logging: RuleLogging,
}

impl FirewallT for Firewall {
    type Error = Error;

    fn new() -> Result<Self> {
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

    fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<()> {
        self.enable()?;
        self.add_anchor()?;
        self.set_rules(policy)
    }

    fn reset_policy(&mut self) -> Result<()> {
        vec![
            self.remove_rules(),
            self.remove_anchor(),
            self.restore_state(),
        ]
        .into_iter()
        .collect::<Result<Vec<_>>>()
        .map(|_| ())
    }
}

impl Firewall {
    fn set_rules(&mut self, policy: FirewallPolicy) -> Result<()> {
        let mut new_filter_rules = vec![];

        new_filter_rules.append(&mut self.get_allow_loopback_rules()?);
        new_filter_rules.append(&mut self.get_allow_dhcp_rules()?);
        new_filter_rules.append(&mut self.get_policy_specific_rules(policy)?);

        let drop_all_rule = self
            .create_rule_builder(FilterRuleAction::Drop)
            .quick(true)
            .build()?;
        new_filter_rules.push(drop_all_rule);

        let mut anchor_change = pfctl::AnchorChange::new();
        anchor_change.set_filter_rules(new_filter_rules);
        Ok(self.pf.set_rules(ANCHOR_NAME, anchor_change)?)
    }

    fn get_policy_specific_rules(
        &mut self,
        policy: FirewallPolicy,
    ) -> Result<Vec<pfctl::FilterRule>> {
        match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                allow_lan,
                pingable_hosts,
            } => {
                let mut rules = vec![self.get_allow_relay_rule(peer_endpoint)?];
                rules.extend(self.get_allow_pingable_hosts(&pingable_hosts)?);
                if allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }
                Ok(rules)
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
            } => {
                let mut rules = vec![];
                let allow_tcp_dns_to_relay_rule = self
                    .create_rule_builder(FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .interface(&tunnel.interface)
                    .proto(pfctl::Proto::Tcp)
                    .to(pfctl::Endpoint::new(tunnel.ipv4_gateway, 53))
                    .build()?;
                rules.push(allow_tcp_dns_to_relay_rule);
                let allow_udp_dns_to_relay_rule = self
                    .create_rule_builder(FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .interface(&tunnel.interface)
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Endpoint::new(tunnel.ipv4_gateway, 53))
                    .build()?;
                rules.push(allow_udp_dns_to_relay_rule);

                if let Some(ipv6_gateway) = tunnel.ipv6_gateway {
                    let v6_dns_rule_tcp = self
                        .create_rule_builder(FilterRuleAction::Pass)
                        .direction(pfctl::Direction::Out)
                        .quick(true)
                        .interface(&tunnel.interface)
                        .proto(pfctl::Proto::Tcp)
                        .to(pfctl::Endpoint::new(ipv6_gateway, 53))
                        .build()?;
                    rules.push(v6_dns_rule_tcp);
                    let v6_dns_rule_udp = self
                        .create_rule_builder(FilterRuleAction::Pass)
                        .direction(pfctl::Direction::Out)
                        .quick(true)
                        .interface(&tunnel.interface)
                        .proto(pfctl::Proto::Udp)
                        .to(pfctl::Endpoint::new(ipv6_gateway, 53))
                        .build()?;
                    rules.push(v6_dns_rule_udp);
                }

                let block_tcp_dns_rule = self
                    .create_rule_builder(FilterRuleAction::Drop)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .proto(pfctl::Proto::Tcp)
                    .to(pfctl::Port::from(53))
                    .build()?;
                rules.push(block_tcp_dns_rule);
                let block_udp_dns_rule = self
                    .create_rule_builder(FilterRuleAction::Drop)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Port::from(53))
                    .build()?;

                rules.push(block_udp_dns_rule);
                rules.push(self.get_allow_relay_rule(peer_endpoint)?);
                rules.push(self.get_allow_tunnel_rule(tunnel.interface.as_str())?);

                if allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }

                Ok(rules)
            }
            FirewallPolicy::Blocked { allow_lan } => {
                let mut rules = Vec::new();
                if allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }
                Ok(rules)
            }
        }
    }

    fn get_allow_relay_rule(&self, relay_endpoint: net::Endpoint) -> Result<pfctl::FilterRule> {
        let pfctl_proto = as_pfctl_proto(relay_endpoint.protocol);

        Ok(self
            .create_rule_builder(FilterRuleAction::Pass)
            .direction(pfctl::Direction::Out)
            .to(relay_endpoint.address)
            .proto(pfctl_proto)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()?)
    }

    fn get_allow_pingable_hosts(
        &self,
        pingable_hosts: &[IpAddr],
    ) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = vec![];
        for host in pingable_hosts.iter() {
            let icmp_proto = match &host {
                IpAddr::V4(_) => pfctl::Proto::Icmp,
                IpAddr::V6(_) => pfctl::Proto::IcmpV6,
            };

            let out_rule = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::Out)
                .to(pfctl::Endpoint::new(*host, 0))
                .proto(icmp_proto)
                .keep_state(pfctl::StatePolicy::Keep)
                .quick(true)
                .build()?;
            rules.push(out_rule);

            let in_rule = self
                .create_rule_builder(FilterRuleAction::Pass)
                .direction(pfctl::Direction::In)
                .from(pfctl::Endpoint::new(*host, 0))
                .proto(icmp_proto)
                .keep_state(pfctl::StatePolicy::Keep)
                .quick(true)
                .build()?;
            rules.push(in_rule);
        }
        Ok(rules)
    }

    fn get_allow_tunnel_rule(&self, tunnel_interface: &str) -> Result<pfctl::FilterRule> {
        Ok(self
            .create_rule_builder(FilterRuleAction::Pass)
            .interface(tunnel_interface)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()?)
    }

    fn get_allow_loopback_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let lo0_rule = self
            .create_rule_builder(FilterRuleAction::Pass)
            .interface("lo0")
            .keep_state(pfctl::StatePolicy::Keep)
            .quick(true)
            .build()?;
        Ok(vec![lo0_rule])
    }

    fn get_allow_lan_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = vec![];
        // IPv4
        for net in &*super::PRIVATE_NETS {
            let mut rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
            rule_builder
                .quick(true)
                .af(pfctl::AddrFamily::Ipv4)
                .from(pfctl::Ip::from(*net));
            let allow_net = rule_builder.to(pfctl::Ip::from(*net)).build()?;
            let allow_multicast = rule_builder
                .to(pfctl::Ip::from(*super::MULTICAST_NET))
                .build()?;
            let allow_ssdp = rule_builder.to(pfctl::Ip::from(*super::SSDP_IP)).build()?;
            rules.push(allow_net);
            rules.push(allow_multicast);
            rules.push(allow_ssdp);
        }
        // IPv6
        let mut rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
        rule_builder
            .quick(true)
            .af(pfctl::AddrFamily::Ipv6)
            .from(pfctl::Ip::from(*super::LOCAL_INET6_NET));
        let allow_net_v6 = rule_builder
            .to(pfctl::Ip::from(*super::LOCAL_INET6_NET))
            .build()?;
        let allow_multicast_v6 = rule_builder
            .to(pfctl::Ip::from(*super::MULTICAST_INET6_NET))
            .build()?;
        rules.push(allow_net_v6);
        rules.push(allow_multicast_v6);

        Ok(rules)
    }

    fn get_allow_dhcp_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let server_port_v4 = pfctl::Port::from(67);
        let client_port_v4 = pfctl::Port::from(68);
        let server_port_v6 = pfctl::Port::from(547);
        let client_port_v6 = pfctl::Port::from(546);
        let mut dhcp_rule_builder = self.create_rule_builder(FilterRuleAction::Pass);
        dhcp_rule_builder.quick(true).proto(pfctl::Proto::Udp);

        let mut rules = Vec::new();
        let allow_outgoing_dhcp_v4 = dhcp_rule_builder
            .direction(pfctl::Direction::Out)
            .from(client_port_v4)
            .to(pfctl::Endpoint::new(Ipv4Addr::BROADCAST, server_port_v4))
            .build()?;
        rules.push(allow_outgoing_dhcp_v4);
        let allow_incoming_dhcp_v4 = dhcp_rule_builder
            .af(pfctl::AddrFamily::Ipv4)
            .direction(pfctl::Direction::In)
            .from(server_port_v4)
            .to(client_port_v4)
            .build()?;
        rules.push(allow_incoming_dhcp_v4);

        for dhcpv6_server in &*super::DHCPV6_SERVER_ADDRS {
            let allow_outgoing_dhcp_v6 = dhcp_rule_builder
                .af(pfctl::AddrFamily::Ipv6)
                .direction(pfctl::Direction::Out)
                .from(pfctl::Endpoint::new(
                    *super::LOCAL_INET6_NET,
                    client_port_v6,
                ))
                .to(pfctl::Endpoint::new(*dhcpv6_server, server_port_v6))
                .build()?;
            rules.push(allow_outgoing_dhcp_v6);
        }
        let allow_incoming_dhcp_v6 = dhcp_rule_builder
            .af(pfctl::AddrFamily::Ipv6)
            .direction(pfctl::Direction::In)
            .from(pfctl::Endpoint::new(
                *super::LOCAL_INET6_NET,
                server_port_v6,
            ))
            .to(pfctl::Endpoint::new(
                *super::LOCAL_INET6_NET,
                client_port_v6,
            ))
            .build()?;
        rules.push(allow_incoming_dhcp_v6);

        Ok(rules)
    }

    fn create_rule_builder(&self, action: FilterRuleAction) -> pfctl::FilterRuleBuilder {
        let mut builder = pfctl::FilterRuleBuilder::default();
        builder.action(action);
        let rule_log = pfctl::RuleLog::IncludeMatchingState;
        if (self.rule_logging == RuleLogging::Pass && action == FilterRuleAction::Pass)
            || (self.rule_logging == RuleLogging::Drop && action == FilterRuleAction::Drop)
            || self.rule_logging == RuleLogging::All
        {
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
            self.pf_was_enabled = Some(self.pf.is_enabled()?);
        }
        Ok(self.pf.try_enable()?)
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
