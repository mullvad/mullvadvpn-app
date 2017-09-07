use super::{Firewall, SecurityPolicy};
use net;
use pfctl;

// alias used to instantiate firewall implementation
pub type ConcreteFirewall = PacketFilter;
pub use pfctl::{Error, ErrorKind, Result};

const ANCHOR_NAME: &'static str = "talpid_core";

impl From<net::TransportProtocol> for pfctl::Proto {
    fn from(protocol: net::TransportProtocol) -> Self {
        match protocol {
            net::TransportProtocol::Udp => pfctl::Proto::Udp,
            net::TransportProtocol::Tcp => pfctl::Proto::Tcp,
        }
    }
}

pub struct PacketFilter {
    pf: pfctl::PfCtl,
    pf_was_enabled: Option<bool>,
}

impl Firewall<Error> for PacketFilter {
    fn new() -> Result<Self> {
        Ok(
            PacketFilter {
                pf: pfctl::PfCtl::new()?,
                pf_was_enabled: None,
            },
        )
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
        ]
                .into_iter()
                .collect::<Result<Vec<_>>>()
                .map(|_| ())
    }
}

impl PacketFilter {
    fn set_rules(&mut self, policy: SecurityPolicy) -> Result<()> {
        let drop_all_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Drop)
            .quick(true)
            .build()?;
        let allow_dns_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .direction(pfctl::Direction::Out)
            .quick(true)
            .to(pfctl::Port::One(53, pfctl::PortUnaryModifier::Equal))
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .build()?;
        let mut new_rules = self.get_loopback_rules()?;

        match policy {
            SecurityPolicy::Connecting(relay_endpoint) => {
                new_rules.push(Self::get_relay_rule(relay_endpoint)?);
            }
            SecurityPolicy::Connected(relay_endpoint, tunnel_interface) => {
                new_rules.push(Self::get_relay_rule(relay_endpoint)?);
                new_rules.push(Self::get_tunnel_rule(tunnel_interface)?);
            }
        };

        new_rules.push(allow_dns_rule);
        new_rules.push(drop_all_rule);

        self.pf.set_rules(ANCHOR_NAME, &new_rules)
    }

    fn get_relay_rule(relay_endpoint: net::Endpoint) -> Result<pfctl::FilterRule> {
        pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .direction(pfctl::Direction::Out)
            .to(relay_endpoint.address)
            .proto(relay_endpoint.protocol)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()
    }

    fn get_tunnel_rule(tunnel_interface: String) -> Result<pfctl::FilterRule> {
        pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .interface(tunnel_interface)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .quick(true)
            .build()
    }

    fn get_loopback_rules(&self) -> Result<Vec<pfctl::FilterRule>> {
        let lo0_rule = pfctl::FilterRuleBuilder::default()
            .action(pfctl::FilterRuleAction::Pass)
            .interface("lo0")
            .keep_state(pfctl::StatePolicy::Keep)
            .quick(true)
            .build()?;
        Ok(vec![lo0_rule])
    }

    fn get_tcp_flags() -> pfctl::TcpFlags {
        pfctl::TcpFlags::new(
            &[pfctl::TcpFlag::Syn],
            &[pfctl::TcpFlag::Syn, pfctl::TcpFlag::Ack],
        )
    }

    fn remove_rules(&mut self) -> Result<()> {
        // remove_anchor() does not deactivate active rules
        self.pf.flush_rules(ANCHOR_NAME, pfctl::RulesetKind::Filter)
    }

    fn enable(&mut self) -> Result<()> {
        if self.pf_was_enabled.is_none() {
            self.pf_was_enabled = Some(self.pf.is_enabled()?);
        }
        self.pf.try_enable()
    }

    fn restore_state(&mut self) -> Result<()> {
        match self.pf_was_enabled.take() {
            Some(true) => self.pf.try_enable(),
            Some(false) => self.pf.try_disable(),
            None => Ok(()),
        }
    }

    fn add_anchor(&mut self) -> Result<()> {
        self.pf.try_add_anchor(ANCHOR_NAME, pfctl::AnchorKind::Filter)
    }

    fn remove_anchor(&mut self) -> Result<()> {
        self.pf.try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Filter)
    }
}
