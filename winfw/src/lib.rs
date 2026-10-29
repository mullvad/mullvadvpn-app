//! Windows firewall for the Mullvad app.
//!
//! A from-scratch Rust replacement of the C++ `winfw` library in
//! `windows/winfw`, meant to be linked directly into the daemon. For every
//! policy the complete set of WFP objects it installs is kept identical to the
//! C++ implementation, guarded by the snapshot tests in `tests/snapshot.rs`.
//!
//! While the replacement is underway, [`cpp_ffi`] contains runtime bindings to
//! the C++ library, used by the snapshot tests as the reference behavior.
//!
//! The module is a singleton per process, like the C++ library: initialize
//! [`Manager::new`] once, then drive it with the `apply_policy_*` methods.

#![cfg(target_os = "windows")]

pub mod context;
pub mod cpp_ffi;
pub mod error;
pub mod guids;
pub mod objects;
pub mod rules;
pub mod session;
mod sessions;

use std::{io, net::IpAddr, time::Duration};

pub use context::Policy;
pub use error::{ApplyError, Error};

/// The rule-composition side of `fwcontext.cpp`.
mod policy_rules {
    use std::net::IpAddr;

    use super::{
        Settings,
        rules::{AllowedTunnelTraffic, RelayEndpoint, Rule, Sublayer},
    };

    /// A relay server, or any other endpoint the firewall permits traffic to.
    #[derive(Clone, Debug)]
    pub struct AllowedEndpoint {
        /// Processes allowed to reach the endpoint; empty means any process.
        pub clients: Vec<String>,
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
        pub clients: Vec<String>,
        /// What traffic is allowed inside the tunnel while connecting.
        pub allowed_tunnel_traffic: AllowedTunnelTraffic,
    }

    /// `FwContext::composePolicyBlocked`.
    pub fn blocked(settings: &Settings, allowed_endpoint: Option<&AllowedEndpoint>) -> Vec<Rule> {
        let mut ruleset = Vec::new();
        append_net_blocked_rules(&mut ruleset);
        append_settings_rules(&mut ruleset, settings);
        append_allowed_endpoint_rules(&mut ruleset, allowed_endpoint);
        ruleset
    }

    /// The ruleset `FwContext::applyPolicyConnecting` assembles.
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
        if let Some(alias) = &tunnel.alias {
            let endpoints = match &tunnel.allowed_tunnel_traffic {
                AllowedTunnelTraffic::None => None,
                AllowedTunnelTraffic::All => None,
                AllowedTunnelTraffic::One(endpoint) => Some((endpoint.clone(), None)),
                AllowedTunnelTraffic::Two(first, second) => {
                    Some((first.clone(), Some(second.clone())))
                }
            };
            match &tunnel.allowed_tunnel_traffic {
                AllowedTunnelTraffic::None => {}
                _ => {
                    for outbound in [true, false] {
                        ruleset.push(tunnel_rule(
                            alias,
                            tunnel.clients.clone(),
                            endpoints.clone(),
                            tunnel.exit_ip,
                            outbound,
                        ));
                    }
                }
            }
        }
        ruleset
    }

    /// The ruleset `FwContext::applyPolicyConnected` assembles.
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
            for outbound in [true, false] {
                ruleset.push(tunnel_rule(
                    alias,
                    tunnel.clients.clone(),
                    None,
                    tunnel.exit_ip,
                    outbound,
                ));
            }
        }
        ruleset
    }

    fn tunnel_rule(
        alias: &str,
        clients: Vec<String>,
        endpoints: Option<(RelayEndpoint, Option<RelayEndpoint>)>,
        exit_ip: Option<IpAddr>,
        outbound: bool,
    ) -> Rule {
        if outbound {
            Rule::PermitVpnTunnel {
                clients,
                alias: alias.to_owned(),
                endpoints,
                exit_ip,
            }
        } else {
            Rule::PermitVpnTunnelService {
                clients,
                alias: alias.to_owned(),
                endpoints,
                exit_ip,
            }
        }
    }

    /// `AppendNetBlockedRules`.
    fn append_net_blocked_rules(ruleset: &mut Vec<Rule>) {
        ruleset.push(Rule::BlockAll);
        ruleset.push(Rule::PermitLoopback);
    }

    /// `AppendSettingsRules`.
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

    /// `AppendRelayRules`.
    fn append_relay_rules(ruleset: &mut Vec<Rule>, relays: &[RelayEndpoint], clients: &[String]) {
        for relay in relays {
            // A relay serving on the DNS port is permitted in the DNS sublayer,
            // which would otherwise block it. See the comment on
            // `AppendSettingsRules` in `fwcontext.cpp`.
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

    /// `AppendAllowedEndpointRules`.
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
/// `WinFwSettings` in `winfw.h`.
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

/// How to clean up when deinitializing, `WINFW_CLEANUP_POLICY` in `winfw.h`.
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

/// The policy currently in effect, `WINFW_ACTIVE_POLICY` in `winfw.h`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActivePolicy {
    #[default]
    None,
    Connecting,
    Connected,
    Blocked,
}

impl From<Policy> for ActivePolicy {
    fn from(policy: Policy) -> Self {
        match policy {
            Policy::None => Self::None,
            Policy::Connecting => Self::Connecting,
            Policy::Connected => Self::Connected,
            Policy::Blocked => Self::Blocked,
        }
    }
}

/// The firewall, `WinFw_Initialize` onward.
///
/// Must be initialized once per process; a second manager would find the
/// first one's provider and sublayer objects already installed, which the
/// initialization transaction removes and re-adds.
pub struct Manager {
    context: Option<context::FwContext>,
}

impl Manager {
    /// `WinFw_Initialize`.
    pub fn new(timeout: Duration) -> Result<Self, Error> {
        context::FwContext::new(timeout)
            .map(|context| Self {
                context: Some(context),
            })
            .map_err(Error::Initialization)
    }

    /// `WinFw_InitializeBlocked`.
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
        self.context.as_mut().ok_or_else(|| {
            ApplyError::from_io(io::Error::other("the firewall module is not initialized"))
        })
    }

    /// `WinFw_ApplyPolicyConnecting`.
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

    /// `WinFw_ApplyPolicyConnected`.
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
            return Err(ApplyError::from_io(io::Error::other(
                "the connected policy requires a tunnel interface",
            )));
        }
        log::debug!("Non-tunnel DNS servers: {}", join_ips(non_tunnel_dns_servers));
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

    /// `WinFw_ApplyPolicyBlocked`.
    pub fn apply_policy_blocked(
        &mut self,
        settings: &Settings,
        allowed_endpoint: Option<&AllowedEndpoint>,
    ) -> Result<(), ApplyError> {
        let context = self.context()?;
        let ruleset = policy_rules::blocked(settings, allowed_endpoint);
        context
            .apply_policy_blocked(&ruleset)
            .map_err(policy_error)
    }

    /// `WinFw_Reset`: clear the policy in effect. Without an initialized
    /// module, this removes everything the firewall ever registered.
    pub fn reset(&mut self) -> Result<(), ApplyError> {
        match self.context.as_mut() {
            Some(context) => context.reset().map_err(policy_error),
            None => purge_all(),
        }
    }

    /// `WinFw_Deinitialize`.
    ///
    /// The context is dropped before the reset, closing the engine session, so
    /// the reset removes everything instead of just reverting the policy: this
    /// mirrors the C++ implementation, where `WinFw_Deinitialize` first
    /// deletes the context and then calls an uninitialized `WinFw_Reset`.
    pub fn deinitialize(&mut self, cleanup: CleanupPolicy) -> Result<(), Error> {
        let Some(context) = self.context.take() else {
            return Ok(());
        };
        let was_blocked = context.active_policy() == Policy::Blocked;
        drop(context);

        if cleanup == CleanupPolicy::BlockingUntilReboot && was_blocked {
            return Ok(());
        }
        context::purge_standalone()
            .map_err(|error| Error::Deinitialization(ApplyError::from_io(error).into_io()))
    }

    /// `WinFw_ActivePolicy`.
    pub fn active_policy(&self) -> ActivePolicy {
        self.context
            .as_ref()
            .map_or(Policy::None, context::FwContext::active_policy)
            .into()
    }

    /// The relay validation of `WinFw_ApplyPolicyConnecting` and
    /// `WinFw_ApplyPolicyConnected`: there must be relays, and none of them
    /// may be the exit endpoint.
    fn validate_relays(
        &self,
        relays: &[RelayEndpoint],
        exit_ip: Option<IpAddr>,
    ) -> Result<(), ApplyError> {
        if relays.is_empty() {
            return Err(ApplyError::from_io(io::Error::other("no relays specified")));
        }

        if let Some(exit_ip) = exit_ip
            && relays.iter().any(|relay| relay.ip == exit_ip)
        {
            return Err(ApplyError::from_io(io::Error::other(
                "relay IP must not equal the exit endpoint IP",
            )));
        }
        Ok(())
    }
}

/// `WinFw_Reset` when the module is not initialized: remove everything the
/// firewall ever registered, in its own engine session.
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
