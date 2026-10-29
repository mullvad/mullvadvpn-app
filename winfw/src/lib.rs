//! Windows firewall for the Mullvad app.
//!
//! The module is a singleton per process: initialize [`Manager::new`] once,
//! then drive it with the `apply_policy_*` methods.

#![cfg(target_os = "windows")]

pub mod context;
pub mod error;
pub mod guids;
pub mod objects;
pub mod rules;
pub mod session;
mod sessions;

use std::{io, net::IpAddr, time::Duration};

pub use error::{ApplyError, Error};

/// The rulesets each policy is composed of.
mod policy_rules {
    use std::{net::IpAddr, path::PathBuf};

    use super::{
        Settings,
        rules::{AllowedTunnelTraffic, RelayEndpoint, Rule, Sublayer},
    };

    /// A relay server, or any other endpoint the firewall permits traffic to.
    #[derive(Clone, Debug)]
    pub struct AllowedEndpoint {
        /// Processes allowed to reach the endpoint; empty means any process.
        pub clients: Vec<PathBuf>,
        pub endpoint: RelayEndpoint,
    }

    /// The relays and tunnel data of a connecting or connected firewall state.
    #[derive(Clone, Debug, Default)]
    pub struct TunnelParameters {
        /// The tunnel interface alias, `Some` only while a tunnel exists.
        pub alias: Option<String>,
        /// The IP of the exit relay in a multihop tunnel, if any.
        pub exit_ip: Option<IpAddr>,
        /// The processes allowed to reach the relays; empty means any process.
        pub clients: Vec<PathBuf>,
        /// What traffic is allowed inside the tunnel while connecting.
        pub allowed_tunnel_traffic: AllowedTunnelTraffic,
    }

    /// The ruleset of the blocked policy.
    pub fn blocked(settings: &Settings, allowed_endpoint: Option<&AllowedEndpoint>) -> Vec<Rule> {
        let mut ruleset = Vec::new();
        append_net_blocked_rules(&mut ruleset);
        append_settings_rules(&mut ruleset, settings);
        append_allowed_endpoint_rules(&mut ruleset, allowed_endpoint);
        ruleset
    }

    /// The ruleset of the connecting policy.
    pub fn connecting(
        settings: &Settings,
        relays: &[RelayEndpoint],
        tunnel: &TunnelParameters,
        allowed_endpoint: &AllowedEndpoint,
    ) -> Vec<Rule> {
        let mut ruleset = Vec::new();
        append_net_blocked_rules(&mut ruleset);
        append_settings_rules(&mut ruleset, settings);
        append_relay_rules(&mut ruleset, relays, &tunnel.clients);
        append_allowed_endpoint_rules(&mut ruleset, Some(allowed_endpoint));

        // The tunnel rules are only installed while the tunnel exists, and
        // the "None" case of the allowed tunnel traffic adds nothing.
        let Some(alias) = &tunnel.alias else {
            return ruleset;
        };
        let endpoints = match &tunnel.allowed_tunnel_traffic {
            AllowedTunnelTraffic::None => return ruleset,
            AllowedTunnelTraffic::All => None,
            AllowedTunnelTraffic::One(endpoint) => Some((endpoint.clone(), None)),
            AllowedTunnelTraffic::Two(first, second) => Some((first.clone(), Some(second.clone()))),
        };
        ruleset.push(tunnel_rule_out(
            alias,
            tunnel.clients.clone(),
            endpoints.clone(),
            tunnel.exit_ip,
        ));
        ruleset.push(tunnel_rule_in(
            alias,
            tunnel.clients.clone(),
            endpoints,
            tunnel.exit_ip,
        ));
        ruleset
    }

    /// The ruleset of the connected policy.
    pub fn connected(
        settings: &Settings,
        relays: &[RelayEndpoint],
        tunnel: &TunnelParameters,
        tunnel_dns_servers: &[IpAddr],
        non_tunnel_dns_servers: &[IpAddr],
    ) -> Vec<Rule> {
        let mut ruleset = Vec::new();
        append_net_blocked_rules(&mut ruleset);
        append_settings_rules(&mut ruleset, settings);
        append_relay_rules(&mut ruleset, relays, &tunnel.clients);

        if let Some(alias) = &tunnel.alias {
            if !tunnel_dns_servers.is_empty() {
                ruleset.push(Rule::DnsPermitTunnel {
                    alias: alias.clone(),
                    hosts: tunnel_dns_servers.to_vec(),
                });
            }
            if !non_tunnel_dns_servers.is_empty() {
                ruleset.push(Rule::DnsPermitNonTunnel {
                    alias: Some(alias.clone()),
                    hosts: non_tunnel_dns_servers.to_vec(),
                });
            }
            ruleset.push(tunnel_rule_out(
                alias,
                tunnel.clients.clone(),
                None,
                tunnel.exit_ip,
            ));
            ruleset.push(tunnel_rule_in(
                alias,
                tunnel.clients.clone(),
                None,
                tunnel.exit_ip,
            ));
        }
        ruleset
    }

    /// Permit outbound traffic inside the tunnel.
    fn tunnel_rule_out(
        alias: &str,
        clients: Vec<PathBuf>,
        endpoints: Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    ) -> Rule {
        Rule::PermitVpnTunnel {
            clients,
            alias: alias.to_owned(),
            endpoints,
            exit_ip,
        }
    }

    /// Permit inbound traffic inside the tunnel.
    fn tunnel_rule_in(
        alias: &str,
        clients: Vec<PathBuf>,
        endpoints: Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
    ) -> Rule {
        Rule::PermitVpnTunnelService {
            clients,
            alias: alias.to_owned(),
            endpoints,
            exit_ip,
        }
    }

    fn append_net_blocked_rules(ruleset: &mut Vec<Rule>) {
        ruleset.push(Rule::BlockAll);
        ruleset.push(Rule::PermitLoopback);
    }

    fn append_settings_rules(ruleset: &mut Vec<Rule>, settings: &Settings) {
        if settings.permit_dhcp {
            ruleset.push(Rule::PermitDhcp);
            ruleset.push(Rule::PermitNdp);
        }

        if settings.permit_lan {
            ruleset.push(Rule::PermitLan);
            ruleset.push(Rule::PermitLanService);
            ruleset.push(Rule::PermitDhcpServer);
        }

        // DNS management: all DNS is lifted out of the baseline sublayer and
        // restricted in the DNS sublayer instead.
        ruleset.push(Rule::PermitDns);
        ruleset.push(Rule::DnsPermitLoopback);
        ruleset.push(Rule::DnsBlockAll);
    }

    fn append_relay_rules(ruleset: &mut Vec<Rule>, relays: &[RelayEndpoint], clients: &[PathBuf]) {
        for relay in relays {
            // A relay serving on the DNS port is permitted in the DNS sublayer:
            // `PermitDns` lifts all DNS out of the baseline sublayer, so the DNS
            // sublayer would otherwise block it.
            let sublayer = if relay.port == 53 {
                Sublayer::Dns
            } else {
                Sublayer::Baseline
            };

            ruleset.push(Rule::PermitEndpoint {
                endpoint: relay.clone(),
                clients: clients.to_vec(),
                sublayer,
            });
        }
    }

    fn append_allowed_endpoint_rules(
        ruleset: &mut Vec<Rule>,
        allowed_endpoint: Option<&AllowedEndpoint>,
    ) {
        if let Some(allowed_endpoint) = allowed_endpoint {
            let sublayer = if allowed_endpoint.endpoint.port == 53 {
                Sublayer::Dns
            } else {
                Sublayer::Baseline
            };
            ruleset.push(Rule::PermitEndpoint {
                endpoint: allowed_endpoint.endpoint.clone(),
                clients: allowed_endpoint.clients.clone(),
                sublayer,
            });
        }
    }
}

pub use policy_rules::{AllowedEndpoint, TunnelParameters};
pub use rules::{AllowedTunnelTraffic, Protocol, RelayEndpoint};

/// Whether DHCP requests are permitted, and private networks are reachable.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub permit_dhcp: bool,
    pub permit_lan: bool,
}

impl Settings {
    /// The settings the daemon uses: DHCP always permitted, LANs optional.
    pub fn new(permit_lan: bool) -> Self {
        Self {
            permit_dhcp: true,
            permit_lan,
        }
    }
}

/// How to clean up when deinitializing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupPolicy {
    /// Remove all objects that have been registered with WFP.
    ResetFirewall,
    /// Continue blocking if this is the active policy. The blocking filters
    /// are ephemeral: they are active until the machine is rebooted. To keep
    /// blocking across a reboot, the caller is responsible for replacing them
    /// with persistent ones.
    BlockingUntilReboot,
}

/// The policy currently in effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActivePolicy {
    #[default]
    None,
    Connecting,
    Connected,
    Blocked,
}

/// The firewall.
///
/// Must be initialized once per process; a second manager would find the
/// first one's provider and sublayer objects already installed, which the
/// initialization transaction removes and re-adds.
pub struct Manager {
    context: Option<context::FwContext>,
}

impl Manager {
    /// Initialize the firewall, with no policy in effect.
    pub fn new(timeout: Duration) -> Result<Self, Error> {
        context::FwContext::new(timeout)
            .map(|context| Self {
                context: Some(context),
            })
            .map_err(Error::Initialization)
    }

    /// Initialize the firewall with the blocked policy in effect, applied in
    /// the initialization transaction.
    pub fn new_blocked(
        timeout: Duration,
        settings: &Settings,
        allowed_endpoint: Option<&AllowedEndpoint>,
    ) -> Result<Self, Error> {
        let ruleset = policy_rules::blocked(settings, allowed_endpoint);
        context::FwContext::new_blocked(timeout, &ruleset)
            .map(|context| Self {
                context: Some(context),
            })
            .map_err(Error::Initialization)
    }

    fn context(&mut self) -> Result<&mut context::FwContext, ApplyError> {
        self.context
            .as_mut()
            .ok_or_else(|| policy_error(io::Error::other("the firewall module is not initialized")))
    }

    /// Apply the connecting policy.
    pub fn apply_policy_connecting(
        &mut self,
        settings: &Settings,
        relays: &[RelayEndpoint],
        tunnel: &TunnelParameters,
        allowed_endpoint: &AllowedEndpoint,
    ) -> Result<(), ApplyError> {
        self.validate_relays(relays, tunnel.exit_ip)?;
        let context = self.context()?;
        let ruleset = policy_rules::connecting(settings, relays, tunnel, allowed_endpoint);
        context
            .apply_policy_connecting(&ruleset)
            .map_err(policy_error)
    }

    /// Apply the connected policy.
    pub fn apply_policy_connected(
        &mut self,
        settings: &Settings,
        relays: &[RelayEndpoint],
        tunnel: &TunnelParameters,
        tunnel_dns_servers: &[IpAddr],
        non_tunnel_dns_servers: &[IpAddr],
    ) -> Result<(), ApplyError> {
        self.validate_relays(relays, tunnel.exit_ip)?;
        if tunnel.alias.is_none() {
            return Err(policy_error(io::Error::other(
                "the connected policy requires a tunnel interface",
            )));
        }
        log::debug!(
            "Non-tunnel DNS servers: {}",
            join_ips(non_tunnel_dns_servers)
        );
        log::debug!("Tunnel DNS servers: {}", join_ips(tunnel_dns_servers));
        let context = self.context()?;
        let ruleset = policy_rules::connected(
            settings,
            relays,
            tunnel,
            tunnel_dns_servers,
            non_tunnel_dns_servers,
        );
        context
            .apply_policy_connected(&ruleset)
            .map_err(policy_error)
    }

    /// Apply the blocked policy.
    pub fn apply_policy_blocked(
        &mut self,
        settings: &Settings,
        allowed_endpoint: Option<&AllowedEndpoint>,
    ) -> Result<(), ApplyError> {
        let context = self.context()?;
        let ruleset = policy_rules::blocked(settings, allowed_endpoint);
        context.apply_policy_blocked(&ruleset).map_err(policy_error)
    }

    /// Clear the policy in effect. Without an initialized
    /// module, this removes everything the firewall ever registered.
    pub fn reset(&mut self) -> Result<(), ApplyError> {
        match self.context.as_mut() {
            Some(context) => context.reset().map_err(policy_error),
            None => purge_all(),
        }
    }

    /// Deinitialize the firewall, cleaning up according to `cleanup`.
    ///
    /// The context is dropped before the reset, closing the engine session, so
    /// the reset removes everything instead of just reverting the policy.
    pub fn deinitialize(&mut self, cleanup: CleanupPolicy) -> Result<(), Error> {
        let Some(context) = self.context.take() else {
            return Ok(());
        };
        let was_blocked = context.active_policy() == ActivePolicy::Blocked;
        drop(context);

        if cleanup == CleanupPolicy::BlockingUntilReboot && was_blocked {
            return Ok(());
        }
        context::purge_standalone().map_err(Error::Deinitialization)
    }

    /// The policy currently in effect.
    pub fn active_policy(&self) -> ActivePolicy {
        self.context
            .as_ref()
            .map_or(ActivePolicy::None, context::FwContext::active_policy)
    }

    /// Validate the relays of the connecting and connected policies: there
    /// must be relays, and none of them may be the exit endpoint.
    fn validate_relays(
        &self,
        relays: &[RelayEndpoint],
        exit_ip: Option<IpAddr>,
    ) -> Result<(), ApplyError> {
        if relays.is_empty() {
            return Err(policy_error(io::Error::other("no relays specified")));
        }

        if let Some(exit_ip) = exit_ip
            && relays.iter().any(|relay| relay.ip == exit_ip)
        {
            return Err(policy_error(io::Error::other(
                "relay IP must not equal the exit endpoint IP",
            )));
        }
        Ok(())
    }
}

/// Remove everything the firewall ever registered, in its own engine session.
/// This is what [`Manager::reset`] does when the module is not initialized.
pub fn purge_all() -> Result<(), ApplyError> {
    context::purge_standalone().map_err(policy_error)
}

/// Classify an error from applying or resetting a policy. When the WFP
/// transaction lock timed out, the processes that may be holding it are
/// logged.
fn policy_error(error: io::Error) -> ApplyError {
    let error = ApplyError::from_io(error);
    if let ApplyError::LockTimeout(_) = error {
        sessions::log_active_sessions();
    }
    error
}

fn join_ips(ips: &[IpAddr]) -> String {
    ips.iter()
        .map(IpAddr::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
