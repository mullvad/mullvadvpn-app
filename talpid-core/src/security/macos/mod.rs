extern crate pfctl;
extern crate tokio_core;

use super::{NetworkSecurity, SecurityPolicy};

use ipnetwork::IpNetwork;

use std::net::Ipv4Addr;
use std::path::Path;

use talpid_types::net;

mod dns;

use self::dns::DnsMonitor;

error_chain! {
    links {
        PfCtl(self::pfctl::Error, self::pfctl::ErrorKind) #[doc = "PF error"];
        DnsMonitor(self::dns::Error, self::dns::ErrorKind) #[doc = "DNS error"];
    }
}

/// TODO(linus): This crate is not supposed to be Mullvad-aware. So at some point this should be
/// replaced by allowing the anchor name to be configured from the public API of this crate.
const ANCHOR_NAME: &'static str = "mullvad";

/// The macOS firewall and DNS implementation.
pub struct MacosNetworkSecurity {
    pf: pfctl::PfCtl,
    pf_was_enabled: Option<bool>,
    dns_monitor: DnsMonitor,
}

impl NetworkSecurity for MacosNetworkSecurity {
    type Error = Error;

    fn new(_cache_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(MacosNetworkSecurity {
            pf: pfctl::PfCtl::new()?,
            pf_was_enabled: None,
            dns_monitor: DnsMonitor::new()?,
        })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        self.enable()?;
        self.add_anchor()?;
        self.set_rules(policy)
    }

    fn reset_policy(&mut self) -> Result<()> {
        vec![
            self.remove_rules(),
            self.remove_anchor(),
            self.restore_state(),
            self.restore_dns(),
        ].into_iter()
        .collect::<Result<Vec<_>>>()
        .map(|_| ())
    }
}

impl MacosNetworkSecurity {
    fn set_rules(&mut self, policy: SecurityPolicy) -> Result<()> {
        let mut new_filter_rules = vec![];

        new_filter_rules.append(&mut Self::get_allow_loopback_rules()?);
        new_filter_rules.append(&mut Self::get_allow_dhcp_rules()?);
        new_filter_rules.append(&mut self.get_policy_specific_rules(policy)?);

        let drop_all_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Drop)
            .quick(true)
            .build()?;
        new_filter_rules.push(drop_all_rule);

        let mut anchor_change = pfctl::AnchorChange::new();
        anchor_change.set_filter_rules(new_filter_rules);
        Ok(self.pf.set_rules(ANCHOR_NAME, anchor_change)?)
    }

    fn get_policy_specific_rules(
        &mut self,
        policy: SecurityPolicy,
    ) -> Result<Vec<pfctl::FilterRule>> {
        match policy {
            SecurityPolicy::Connecting {
                relay_endpoint,
                allow_lan,
            } => {
                let mut rules = vec![Self::get_allow_relay_rule(relay_endpoint)?];
                if allow_lan {
                    rules.append(&mut Self::get_allow_lan_rules()?);
                }
                Ok(rules)
            }
            SecurityPolicy::Connected {
                relay_endpoint,
                tunnel,
                allow_lan,
            } => {
                self.dns_monitor.set_dns(vec![tunnel.gateway.to_string()])?;

                let allow_tcp_dns_to_relay_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .interface(&tunnel.interface)
                    .proto(pfctl::Proto::Tcp)
                    .to(pfctl::Endpoint::new(tunnel.gateway, 53))
                    .build()?;
                let allow_udp_dns_to_relay_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .interface(&tunnel.interface)
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Endpoint::new(tunnel.gateway, 53))
                    .build()?;
                let block_tcp_dns_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Drop)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .proto(pfctl::Proto::Tcp)
                    .to(pfctl::Port::from(53))
                    .build()?;
                let block_udp_dns_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Drop)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Port::from(53))
                    .build()?;

                let mut rules = vec![
                    allow_tcp_dns_to_relay_rule,
                    allow_udp_dns_to_relay_rule,
                    block_tcp_dns_rule,
                    block_udp_dns_rule,
                    Self::get_allow_relay_rule(relay_endpoint)?,
                    Self::get_allow_tunnel_rule(tunnel.interface.as_str())?,
                ];

                if allow_lan {
                    rules.append(&mut Self::get_allow_lan_rules()?);
                }
                Ok(rules)
            }
            SecurityPolicy::Blocked { allow_lan } => {
                let mut rules = Vec::new();
                if allow_lan {
                    rules.append(&mut Self::get_allow_lan_rules()?);
                }
                Ok(rules)
            }
        }
    }

    fn get_allow_relay_rule(relay_endpoint: net::Endpoint) -> Result<pfctl::FilterRule> {
        let pfctl_proto = as_pfctl_proto(relay_endpoint.protocol);

        Ok(pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .direction(pfctl::Direction::Out)
            .to(relay_endpoint.address)
            .proto(pfctl_proto)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()?)
    }

    fn get_allow_tunnel_rule(tunnel_interface: &str) -> Result<pfctl::FilterRule> {
        Ok(pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .interface(tunnel_interface)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()?)
    }

    fn get_allow_loopback_rules() -> Result<Vec<pfctl::FilterRule>> {
        let lo0_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .interface("lo0")
            .keep_state(pfctl::StatePolicy::Keep)
            .quick(true)
            .build()?;
        Ok(vec![lo0_rule])
    }

    fn get_allow_lan_rules() -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = vec![];
        for net in &*super::PRIVATE_NETS {
            let mut rule_builder = pfctl::FilterRuleBuilder::default();
            rule_builder
                .action(pfctl::FilterRuleAction::Pass)
                .quick(true)
                .af(pfctl::AddrFamily::Ipv4)
                .from(pfctl::Ip::from(ipnetwork_compat(IpNetwork::V4(*net))));
            let allow_net = rule_builder
                .to(pfctl::Ip::from(ipnetwork_compat(IpNetwork::V4(*net))))
                .build()?;
            let allow_multicast = rule_builder
                .to(pfctl::Ip::from(ipnetwork_compat(IpNetwork::V4(
                    *super::MULTICAST_NET,
                )))).build()?;
            rules.push(allow_net);
            rules.push(allow_multicast);
        }
        Ok(rules)
    }

    fn get_allow_dhcp_rules() -> Result<Vec<pfctl::FilterRule>> {
        let broadcast_address = Ipv4Addr::new(255, 255, 255, 255);
        let server_port = pfctl::Port::from(67);
        let client_port = pfctl::Port::from(68);
        let mut dhcp_rule_builder = pfctl::FilterRuleBuilder::default();
        dhcp_rule_builder
            .action(pfctl::FilterRuleAction::Pass)
            .proto(pfctl::Proto::Udp)
            .quick(true)
            .keep_state(pfctl::StatePolicy::Keep);
        let allow_outgoing_dhcp = dhcp_rule_builder
            .direction(pfctl::Direction::Out)
            .from(client_port)
            .to(pfctl::Endpoint::new(broadcast_address, server_port))
            .build()?;
        let allow_incoming_dhcp = dhcp_rule_builder
            .direction(pfctl::Direction::In)
            .from(server_port)
            .to(client_port)
            .build()?;
        Ok(vec![allow_outgoing_dhcp, allow_incoming_dhcp])
    }

    fn get_tcp_flags() -> pfctl::TcpFlags {
        pfctl::TcpFlags::new(
            &[pfctl::TcpFlag::Syn],
            &[pfctl::TcpFlag::Syn, pfctl::TcpFlag::Ack],
        )
    }

    fn remove_rules(&mut self) -> Result<()> {
        // remove_anchor() does not deactivate active rules
        Ok(self
            .pf
            .flush_rules(ANCHOR_NAME, pfctl::RulesetKind::Filter)?)
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

    fn restore_dns(&self) -> Result<()> {
        Ok(self.dns_monitor.reset()?)
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

/// Converts a network from the struct version that talpid-core uses to the version pfctl uses.
fn ipnetwork_compat(net: ::ipnetwork::IpNetwork) -> pfctl::ipnetwork::IpNetwork {
    pfctl::ipnetwork::IpNetwork::new(net.ip(), net.prefix())
        .expect("IpNetwork versions not compatible")
}
