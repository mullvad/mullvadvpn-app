use super::{FirewallArguments, FirewallPolicy};
use ipnetwork::IpNetwork;
use libc::{c_int, sysctlbyname};
use pfctl::{DropAction, FilterRuleAction, Uid};
use std::{
    env, io,
    net::{IpAddr, Ipv4Addr},
    ptr,
};
use subslice::SubsliceExt;
use talpid_types::net::{
    self, AllowedEndpoint, AllowedTunnelTraffic, ALLOWED_LAN_MULTICAST_NETS, ALLOWED_LAN_NETS,
};

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
        self.set_rules(&policy)?;

        if let Err(error) = self.flush_states(&policy) {
            log::error!("Failed to clear PF connection states: {error}");
        }

        Ok(())
    }

    /// Clear PF connection states. That is, forget connections that were previously approved by a
    /// `pass` rule, and force PF to make new verdicts.
    /// PF retains approved connections forever, even after a responsible anchor or rule has been
    /// removed. Therefore, they should be flushed after every state transition to ensure approved
    /// states conform to our desired policy.
    fn flush_states(&mut self, policy: &FirewallPolicy) -> Result<()> {
        self.pf
            .get_states()?
            .into_iter()
            .filter(|state| {
                // If we can't parse a state for whatever reason, err on the safe side and keep it
                Self::should_delete_state(policy, state).unwrap_or(false)
            })
            .for_each(|state| {
                if let Err(error) = self.pf.kill_state(&state) {
                    log::warn!("Failed to delete PF state: {error}");
                }
            });

        Ok(())
    }

    /// Clearing the VPN server connection seems to interrupt ephemeral key exchange on some
    /// machines, so we kill any state except that one as well as within-tunnel connections that
    /// should still be allowed.
    fn should_delete_state(policy: &FirewallPolicy, state: &pfctl::State) -> Result<bool> {
        let allowed_tunnel_traffic = policy.allowed_tunnel_traffic();
        let tunnel_ips = policy
            .tunnel()
            .map(|tunnel| tunnel.ips.as_slice())
            .unwrap_or_default();

        let local_address = state.local_address()?;
        let remote_address = state.remote_address()?;
        let proto = state.proto()?;

        let Some(peer) = policy.peer_endpoint().map(|endpoint| endpoint.endpoint) else {
            // If there's no peer, there's also no tunnel. We have no states to preserve
            return Ok(true);
        };

        let should_delete = if tunnel_ips.contains(&local_address.ip()) {
            // Tunnel traffic: Clear states except those allowed in the tunnel
            // Ephemeral peer exchange becomes unreliable otherwise, when multihop is enabled
            match allowed_tunnel_traffic {
                AllowedTunnelTraffic::None => true,
                AllowedTunnelTraffic::All => false,
                AllowedTunnelTraffic::One(endpoint) => endpoint.address != remote_address,
                AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
                    endpoint1.address != remote_address && endpoint2.address != remote_address
                }
            }
        } else {
            // Non-tunnel traffic: Clear all states except traffic destined for the VPN endpoint
            // Ephemeral peer exchange becomes unreliable otherwise
            peer.address != remote_address || as_pfctl_proto(peer.protocol) != proto
        };

        Ok(should_delete)
    }

    pub fn reset_policy(&mut self) -> Result<()> {
        // Implemented this way to not early return on an error.
        // We always want all three methods to run, and then return
        // the first error it encountered, if any.
        self.remove_rules()
            .and(self.remove_anchor())
            .and(self.restore_state())
    }

    fn set_rules(&mut self, policy: &FirewallPolicy) -> Result<()> {
        let mut new_filter_rules = vec![];

        new_filter_rules.append(&mut self.get_allow_loopback_rules()?);
        new_filter_rules.append(&mut self.get_allow_dhcp_client_rules()?);
        new_filter_rules.append(&mut self.get_allow_ndp_rules()?);
        new_filter_rules.append(&mut self.get_policy_specific_rules(policy)?);

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
        anchor_change.set_scrub_rules(Self::get_scrub_rules()?);
        anchor_change.set_filter_rules(new_filter_rules);
        anchor_change.set_redirect_rules(self.get_dns_redirect_rules(policy)?);
        self.pf.set_rules(ANCHOR_NAME, anchor_change)
    }

    fn get_scrub_rules() -> Result<Vec<pfctl::ScrubRule>> {
        // Filter only reassembled packets. Without this, PF will filter based on individual
        // fragments, which may not have complete transport-layer headers.
        let scrub_rule = pfctl::ScrubRuleBuilder::default()
            .action(pfctl::ScrubRuleAction::Scrub)
            .build()?;
        Ok(vec![scrub_rule])
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
                redirect_interface,
            } => {
                let mut rules = vec![self.get_allow_relay_rule(peer_endpoint)?];
                rules.push(self.get_allowed_endpoint_rule(allowed_endpoint)?);

                // Important to block DNS after allow relay rule (so the relay can operate
                // over port 53) but before allow LAN (so DNS does not leak to the LAN)
                rules.append(&mut self.get_block_dns_rules()?);

                if let Some(tunnel) = tunnel {
                    match redirect_interface {
                        Some(redirect_interface) => {
                            enable_forwarding();

                            if !allowed_tunnel_traffic.all() {
                                log::warn!("Split tunneling does not respect the 'allowed tunnel traffic' setting");
                            }
                            rules.append(
                                &mut self.get_split_tunnel_rules(
                                    &tunnel.interface,
                                    redirect_interface,
                                )?,
                            );
                        }
                        None => {
                            rules.extend(self.get_allow_tunnel_rules(
                                &tunnel.interface,
                                allowed_tunnel_traffic,
                            )?);
                        }
                    }
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
                dns_config,
                redirect_interface,
            } => {
                let mut rules = vec![];

                for server in &dns_config.tunnel_config {
                    rules.append(
                        &mut self.get_allow_tunnel_dns_rules_when_connected(tunnel, *server)?,
                    );
                }
                for server in &dns_config.non_tunnel_config {
                    rules.append(
                        &mut self.get_allow_local_dns_rules_when_connected(tunnel, *server)?,
                    );
                }

                rules.push(self.get_allow_relay_rule(peer_endpoint)?);

                // Important to block DNS *before* we allow the tunnel and allow LAN. So DNS
                // can't leak to the wrong IPs in the tunnel or on the LAN.
                rules.append(&mut self.get_block_dns_rules()?);

                if *allow_lan {
                    rules.append(&mut self.get_allow_lan_rules()?);
                }

                if let Some(redirect_interface) = redirect_interface {
                    enable_forwarding();

                    rules.append(
                        &mut self.get_split_tunnel_rules(&tunnel.interface, redirect_interface)?,
                    );
                } else {
                    rules.extend(self.get_allow_tunnel_rules(
                        tunnel.interface.as_str(),
                        &AllowedTunnelTraffic::All,
                    )?);
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
                    rules.push(self.get_allowed_endpoint_rule(allowed_endpoint)?);
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

    fn get_allow_local_dns_rules_when_connected(
        &self,
        tunnel: &crate::tunnel::TunnelMetadata,
        server: IpAddr,
    ) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = Vec::with_capacity(4);

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

        Ok(rules)
    }

    fn get_allow_tunnel_dns_rules_when_connected(
        &self,
        tunnel: &crate::tunnel::TunnelMetadata,
        server: IpAddr,
    ) -> Result<Vec<pfctl::FilterRule>> {
        let mut rules = Vec::with_capacity(4);

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

        Ok(rules)
    }

    fn get_allow_relay_rule(
        &self,
        relay_endpoint: &net::AllowedEndpoint,
    ) -> Result<pfctl::FilterRule> {
        let pfctl_proto = as_pfctl_proto(relay_endpoint.endpoint.protocol);

        let mut builder = self.create_rule_builder(FilterRuleAction::Pass);
        builder
            .direction(pfctl::Direction::Out)
            .to(relay_endpoint.endpoint.address)
            .proto(pfctl_proto)
            .keep_state(pfctl::StatePolicy::Keep)
            .quick(true);

        if !relay_endpoint.clients.allow_all() {
            builder.user(Uid::from(super::ROOT_UID));
        }

        builder.build()
    }

    /// Produces a rule that allows traffic to flow to the API. Allows the app (or other apps if
    /// configured) to reach the API in blocked states.
    fn get_allowed_endpoint_rule(
        &self,
        allowed_endpoint: &AllowedEndpoint,
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

    fn get_allow_tunnel_rules(
        &self,
        tunnel_interface: &str,
        allowed_traffic: &AllowedTunnelTraffic,
    ) -> Result<Vec<pfctl::FilterRule>> {
        self.get_allow_tunnel_rules_inner(tunnel_interface, allowed_traffic, Self::get_tcp_flags())
    }

    fn get_allow_tunnel_rules_inner(
        &self,
        tunnel_interface: &str,
        allowed_traffic: &AllowedTunnelTraffic,
        tcp_flags: pfctl::TcpFlags,
    ) -> Result<Vec<pfctl::FilterRule>> {
        let mut base_rule = &mut self.create_rule_builder(FilterRuleAction::Pass);
        base_rule
            .quick(true)
            .interface(tunnel_interface)
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(tcp_flags);

        Ok(match allowed_traffic {
            AllowedTunnelTraffic::One(endpoint) => {
                let pfctl_proto = as_pfctl_proto(endpoint.protocol);
                base_rule = base_rule.to(endpoint.address).proto(pfctl_proto);
                vec![base_rule.build()?]
            }
            AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
                let mut rules = Vec::with_capacity(2);

                let pfctl_proto = as_pfctl_proto(endpoint1.protocol);
                base_rule = base_rule.to(endpoint1.address).proto(pfctl_proto);
                rules.push(base_rule.build()?);

                let pfctl_proto = as_pfctl_proto(endpoint2.protocol);
                base_rule = base_rule.to(endpoint2.address).proto(pfctl_proto);
                rules.push(base_rule.build()?);

                rules
            }
            AllowedTunnelTraffic::All => {
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
        for net in &*ALLOWED_LAN_NETS {
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
        for multicast_net in &*ALLOWED_LAN_MULTICAST_NETS {
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

    fn get_split_tunnel_rules(
        &self,
        from_interface: &str,
        to_interface: &str,
    ) -> Result<Vec<pfctl::FilterRule>> {
        let tunnel_rule = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .direction(pfctl::Direction::In)
            .keep_state(pfctl::StatePolicy::None)
            .interface(from_interface)
            .build()?;
        let allow_rule = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .direction(pfctl::Direction::Out)
            .keep_state(pfctl::StatePolicy::Keep)
            .interface(to_interface)
            .build()?;
        let redir_rule = self
            .create_rule_builder(FilterRuleAction::Pass)
            .quick(true)
            .direction(pfctl::Direction::Out)
            .route(pfctl::Route::RouteTo(pfctl::PoolAddr::from(
                pfctl::Interface::from(to_interface),
            )))
            .keep_state(pfctl::StatePolicy::Keep)
            .tcp_flags(Self::get_tcp_flags())
            .interface(from_interface)
            .build()?;
        Ok(vec![tunnel_rule, allow_rule, redir_rule])
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
        self.pf
            .try_add_anchor(ANCHOR_NAME, pfctl::AnchorKind::Scrub)?;
        Ok(())
    }

    fn remove_anchor(&mut self) -> Result<()> {
        self.pf
            .try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Scrub)?;
        self.pf
            .try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Redirect)?;
        self.pf
            .try_remove_anchor(ANCHOR_NAME, pfctl::AnchorKind::Filter)?;
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

fn enable_forwarding() {
    if let Err(error) = enable_forwarding_for_family(true) {
        log::error!("Failed to enable forwarding (IPv4): {error}");
    }
    if let Err(error) = enable_forwarding_for_family(false) {
        log::error!("Failed to enable forwarding (IPv6): {error}");
    }
}

fn enable_forwarding_for_family(ipv4: bool) -> io::Result<()> {
    if ipv4 {
        log::trace!("Enabling forwarding (IPv4)");
    } else {
        log::trace!("Enabling forwarding (IPv6)");
    }

    let mut val: c_int = 1;

    let option = if ipv4 {
        c"net.inet.ip.forwarding"
    } else {
        c"net.inet6.ip6.forwarding"
    };

    // SAFETY: The strings are null-terminated.
    let result = unsafe {
        sysctlbyname(
            option.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut val as *mut _ as _,
            std::mem::size_of_val(&val),
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result));
    }
    Ok(())
}
