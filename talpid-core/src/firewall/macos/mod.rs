extern crate pfctl;
extern crate tokio_core;

use super::{Firewall, SecurityPolicy};

use std::net::Ipv4Addr;

use talpid_types::net;

mod dns;

use self::dns::DnsMonitor;

error_chain! {
    links {
        PfCtl(self::pfctl::Error, self::pfctl::ErrorKind);
        DnsMonitor(self::dns::Error, self::dns::ErrorKind);
    }
}

// alias used to instantiate firewall implementation
pub type ConcreteFirewall = PacketFilter;

const ANCHOR_NAME: &'static str = "mullvad";

pub struct PacketFilter {
    pf: pfctl::PfCtl,
    pf_was_enabled: Option<bool>,
    dns_monitor: DnsMonitor,
}

impl Firewall<Error> for PacketFilter {
    fn new() -> Result<Self> {
        Ok(PacketFilter {
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
        ].into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|_| ())
    }
}

impl PacketFilter {
    fn set_rules(&mut self, policy: SecurityPolicy) -> Result<()> {
        let mut new_filter_rules = vec![];
        let mut new_redirect_rules = vec![];

        new_filter_rules.append(&mut Self::get_allow_loopback_rules()?);
        new_filter_rules.append(&mut Self::get_allow_dhcp_rules()?);

        let (mut policy_filter_rules, mut policy_redirect_rules) =
            self.get_policy_specific_rules(policy)?;
        new_filter_rules.append(&mut policy_filter_rules);
        new_redirect_rules.append(&mut policy_redirect_rules);

        let drop_all_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Drop)
            .quick(true)
            .build()?;
        new_filter_rules.push(drop_all_rule);

        let mut anchor_change = pfctl::AnchorChange::new();
        anchor_change.set_filter_rules(new_filter_rules);
        anchor_change.set_redirect_rules(new_redirect_rules);
        Ok(self.pf.set_rules(ANCHOR_NAME, anchor_change)?)
    }

    fn get_policy_specific_rules(
        &mut self,
        policy: SecurityPolicy,
    ) -> Result<(Vec<pfctl::FilterRule>, Vec<pfctl::RedirectRule>)> {
        match policy {
            SecurityPolicy::Connecting(relay_endpoint) => {
                Ok((vec![Self::get_allow_relay_rule(relay_endpoint)?], vec![]))
            }
            SecurityPolicy::Connected(relay_endpoint, tunnel) => {
                let allow_dns_to_relay_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .interface(&tunnel.interface)
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Endpoint::new(tunnel.gateway, 53))
                    .build()?;
                let reroute_dns_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Pass)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .route(pfctl::Route::route_to(pfctl::Interface::from("lo0")))
                    .proto(pfctl::Proto::Udp)
                    .to(pfctl::Port::from(53))
                    .build()?;
                let block_all_other_dns_rule = pfctl::FilterRuleBuilder::default()
                    .action(pfctl::FilterRuleAction::Drop)
                    .direction(pfctl::Direction::Out)
                    .quick(true)
                    .proto(pfctl::Proto::Tcp)
                    .to(pfctl::Port::from(53))
                    .build()?;

                Ok((
                    vec![
                        allow_dns_to_relay_rule,
                        reroute_dns_rule,
                        block_all_other_dns_rule,
                        Self::get_allow_relay_rule(relay_endpoint)?,
                        Self::get_allow_tunnel_rule(tunnel.interface.as_str())?,
                    ],
                    vec![],
                ))
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
        Ok(self.pf
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
