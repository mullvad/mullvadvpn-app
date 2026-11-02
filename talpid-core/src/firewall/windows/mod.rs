use std::{net::IpAddr, path::PathBuf, sync::LazyLock, time::Duration};

use talpid_error::ErrorExt;
use talpid_tunnel::TunnelMetadata;
use talpid_types::{
    net::{AllowedEndpoint, AllowedTunnelTraffic, Endpoint, TransportProtocol},
    tunnel::FirewallPolicyError,
};

use super::{FirewallArguments, FirewallPolicy, InitialFirewallState};
use talpid_dns::ResolvedDnsConfig;
use winfw::CleanupPolicy;

mod hyperv;

/// Timeout for acquiring the WFP transaction lock.
const TRANSACTION_TIMEOUT: Duration = Duration::from_secs(5);

const HYPERV_LEAK_WARNING_MSG: &str = "Hyper-V (e.g. WSL machines) may leak in blocked states.";

// `COMLibrary` must be initialized for per thread, so use TLS
thread_local! {
    static WMI: Option<wmi::WMIConnection> = {
        let result = hyperv::init_wmi();
        if matches!(&result, Err(hyperv::Error::ObtainHyperVClass(_))) {
            log::warn!("The Hyper-V firewall is not available. {HYPERV_LEAK_WARNING_MSG}");
            return None;
        }
        consume_and_log_hyperv_err(
            "Initialize COM and WMI",
            result,
        )
    };
}

/// Enable or disable blocking Hyper-V rule
static BLOCK_HYPERV: LazyLock<bool> = LazyLock::new(|| {
    let enable = std::env::var("TALPID_FIREWALL_BLOCK_HYPERV")
        .map(|v| v != "0")
        .unwrap_or(true);

    if !enable {
        log::debug!("Hyper-V block rule disabled by TALPID_FIREWALL_BLOCK_HYPERV");
    }

    enable
});

/// Errors that can happen when configuring the Windows firewall.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error("Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error("Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy(#[source] FirewallPolicyError),

    /// Failure to apply a firewall _connected_ policy
    #[error("Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy(#[source] FirewallPolicyError),

    /// Failure to apply firewall _blocked_ policy
    #[error("Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy(#[source] FirewallPolicyError),

    /// Failure to reset firewall policies
    #[error("Failed to reset firewall policies")]
    ResettingPolicy(#[source] FirewallPolicyError),
}

/// The Windows implementation for the firewall.
pub struct Firewall {
    /// The firewall module.
    // INVARIANT! Must be set until taken when dropped.
    manager: Option<winfw::Manager>,
    /// If firewall rules should even if firewall module is shut down or dies.
    ///
    /// This should only very cautiously be turned off.
    persist: bool,
}

impl Firewall {
    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        if let InitialFirewallState::Blocked(allowed_endpoint) = args.initial_state {
            Self::initialize_blocked(allowed_endpoint, args.allow_lan)
        } else {
            Self::new()
        }
    }

    pub fn new() -> Result<Self, Error> {
        let manager = winfw::Manager::new(TRANSACTION_TIMEOUT)
            .map_err(|error| map_init_error(error, "initialize"))?;
        log::trace!("Successfully initialized windows firewall module");
        Ok(Firewall::with_manager(manager))
    }

    fn initialize_blocked(
        allowed_endpoint: AllowedEndpoint,
        allow_lan: bool,
    ) -> Result<Self, Error> {
        let manager = winfw::Manager::new_blocked(
            TRANSACTION_TIMEOUT,
            &winfw::Settings::new(allow_lan),
            Some(&to_allowed_endpoint(&allowed_endpoint)),
        )
        .map_err(|error| map_init_error(error, "initialize to a blocking state"))?;
        log::trace!("Successfully initialized windows firewall module to a blocking state");

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
        });

        Ok(Firewall::with_manager(manager))
    }

    fn with_manager(manager: winfw::Manager) -> Self {
        Self {
            manager: Some(manager),
            persist: true,
        }
    }

    fn manager(&mut self) -> &mut winfw::Manager {
        self.manager
            .as_mut()
            .expect("the manager is only taken when dropping the firewall")
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        let should_block_hyperv = matches!(
            policy,
            FirewallPolicy::Connecting { .. }
                | FirewallPolicy::Blocked { .. }
                | FirewallPolicy::Disconnecting { .. }
        );

        let apply_result = match policy {
            FirewallPolicy::Connecting {
                peer_endpoints,
                exit_endpoint_ip,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
            } => {
                let cfg = &winfw::Settings::new(allow_lan);
                self.set_connecting_state(
                    &peer_endpoints,
                    exit_endpoint_ip,
                    cfg,
                    tunnel.as_ref(),
                    allowed_endpoint,
                    &allowed_tunnel_traffic,
                )
            }
            FirewallPolicy::Connected {
                peer_endpoints,
                exit_endpoint_ip,
                tunnel,
                allow_lan,
                dns_config,
            } => {
                let cfg = &winfw::Settings::new(allow_lan);
                self.set_connected_state(
                    &peer_endpoints,
                    exit_endpoint_ip,
                    cfg,
                    &tunnel,
                    &dns_config,
                )
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
            } => {
                let cfg = &winfw::Settings::new(allow_lan);
                self.set_blocked_state(cfg, allowed_endpoint)
            }
            // Windows doesn't need to change any existing firewall rules here. Disconnecting state is purely for the Hyper-V blocking which is done later.
            FirewallPolicy::Disconnecting { .. } => Ok(()),
        };

        with_wmi_if_enabled(|wmi| {
            if should_block_hyperv {
                let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
            } else {
                let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
            }
        });

        apply_result
    }

    pub fn reset_policy(&mut self) -> Result<(), Error> {
        self.manager()
            .reset()
            .map_err(map_apply_error("reset the policy of"))
            .map_err(Error::ResettingPolicy)?;

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
        });

        Ok(())
    }

    pub fn persist(&mut self, persist: bool) {
        self.persist = persist;
    }

    fn set_connecting_state(
        &mut self,
        peer_endpoints: &[AllowedEndpoint],
        exit_endpoint_ip: Option<IpAddr>,
        winfw_settings: &winfw::Settings,
        tunnel_metadata: Option<&TunnelMetadata>,
        allowed_endpoint: AllowedEndpoint,
        allowed_tunnel_traffic: &AllowedTunnelTraffic,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connecting' firewall policy");
        let (relays, tunnel_clients) =
            to_relays(peer_endpoints).map_err(Error::ApplyingConnectingPolicy)?;
        let tunnel = winfw::TunnelParameters {
            alias: tunnel_metadata.map(|metadata| metadata.interface.clone()),
            exit_ip: exit_endpoint_ip,
            clients: tunnel_clients,
            allowed_tunnel_traffic: to_allowed_tunnel_traffic(allowed_tunnel_traffic),
        };
        self.manager()
            .apply_policy_connecting(
                winfw_settings,
                &relays,
                &tunnel,
                &to_allowed_endpoint(&allowed_endpoint),
            )
            .map_err(map_apply_error("apply the connecting policy of"))
            .map_err(Error::ApplyingConnectingPolicy)
    }

    fn set_connected_state(
        &mut self,
        peer_endpoints: &[AllowedEndpoint],
        exit_endpoint_ip: Option<IpAddr>,
        winfw_settings: &winfw::Settings,
        tunnel_metadata: &TunnelMetadata,
        dns_config: &ResolvedDnsConfig,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connected' firewall policy");
        let (relays, tunnel_clients) =
            to_relays(peer_endpoints).map_err(Error::ApplyingConnectedPolicy)?;
        let tunnel = winfw::TunnelParameters {
            alias: Some(tunnel_metadata.interface.clone()),
            exit_ip: exit_endpoint_ip,
            clients: tunnel_clients,
            allowed_tunnel_traffic: to_allowed_tunnel_traffic(&AllowedTunnelTraffic::All),
        };
        self.manager()
            .apply_policy_connected(
                winfw_settings,
                &relays,
                &tunnel,
                dns_config.tunnel_config(),
                dns_config.non_tunnel_config(),
            )
            .map_err(map_apply_error("apply the connected policy of"))
            .map_err(Error::ApplyingConnectedPolicy)
    }

    fn set_blocked_state(
        &mut self,
        winfw_settings: &winfw::Settings,
        allowed_endpoint: Option<AllowedEndpoint>,
    ) -> Result<(), Error> {
        log::trace!("Applying 'blocked' firewall policy");
        let winfw_allowed_endpoint = allowed_endpoint.as_ref().map(to_allowed_endpoint);
        self.manager()
            .apply_policy_blocked(winfw_settings, winfw_allowed_endpoint.as_ref())
            .map_err(map_apply_error("apply the blocked policy of"))
            .map_err(Error::ApplyingBlockedPolicy)
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        let Some(manager) = self.manager.take() else {
            return;
        };

        // If the blocked policy is in effect, keep blocking after shutting down. Otherwise,
        // everything is removed.
        let cleanup = if self.persist {
            CleanupPolicy::BlockingPersistent
        } else {
            CleanupPolicy::BlockingUntilReboot
        };

        match manager.deinitialize(cleanup) {
            Ok(()) => log::trace!("Successfully deinitialized windows firewall module"),
            Err(error) => log::error!(
                "{}",
                error.display_chain_with_msg("Failed to deinitialize windows firewall module")
            ),
        }
    }
}

// Convert `result` into an option and log the error, if any.
fn consume_and_log_hyperv_err<T>(
    action: &'static str,
    result: Result<T, hyperv::Error>,
) -> Option<T> {
    result
        .inspect_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg(&format!("{action}. {HYPERV_LEAK_WARNING_MSG}"))
            );
        })
        .ok()
}

// Run a closure with the current thread's WMI connection, if available
fn with_wmi_if_enabled(f: impl FnOnce(&wmi::WMIConnection)) {
    if !*BLOCK_HYPERV {
        return;
    }
    WMI.with(|wmi| {
        if let Some(con) = wmi {
            f(con)
        }
    })
}

/// The endpoint allowed outside the tunnel, in the `winfw` crate's form.
fn to_allowed_endpoint(endpoint: &AllowedEndpoint) -> winfw::AllowedEndpoint {
    winfw::AllowedEndpoint {
        clients: endpoint.clients.to_vec(),
        endpoint: to_relay_endpoint(&endpoint.endpoint),
    }
}

/// Convert the peer endpoints into relays, requiring they allow the same
/// clients, like the previous FFI layer did.
fn to_relays(
    peer_endpoints: &[AllowedEndpoint],
) -> Result<(Vec<winfw::RelayEndpoint>, Vec<PathBuf>), talpid_types::tunnel::FirewallPolicyError> {
    let mut relays = Vec::with_capacity(peer_endpoints.len());
    let mut tunnel_clients: Option<&[PathBuf]> = None;

    for peer in peer_endpoints {
        if let Some(clients) = tunnel_clients {
            if clients != peer.clients.as_ref() {
                log::error!("Relay endpoints must have same allowed clients");
                return Err(talpid_types::tunnel::FirewallPolicyError::Generic);
            }
        } else {
            tunnel_clients = Some(&peer.clients);
        }

        relays.push(to_relay_endpoint(&peer.endpoint));
    }

    let clients = tunnel_clients
        .ok_or_else(|| {
            log::error!("Allowed clients missing. Zero peer endpoints?");
            talpid_types::tunnel::FirewallPolicyError::Generic
        })?
        .to_vec();

    Ok((relays, clients))
}

fn to_relay_endpoint(endpoint: &Endpoint) -> winfw::RelayEndpoint {
    winfw::RelayEndpoint {
        ip: endpoint.address.ip(),
        port: endpoint.address.port(),
        protocol: match endpoint.protocol {
            TransportProtocol::Tcp => winfw::Protocol::Tcp,
            TransportProtocol::Udp => winfw::Protocol::Udp,
        },
    }
}

fn to_allowed_tunnel_traffic(traffic: &AllowedTunnelTraffic) -> winfw::AllowedTunnelTraffic {
    match traffic {
        AllowedTunnelTraffic::None => winfw::AllowedTunnelTraffic::None,
        AllowedTunnelTraffic::All => winfw::AllowedTunnelTraffic::All,
        AllowedTunnelTraffic::One(endpoint) => {
            winfw::AllowedTunnelTraffic::One(to_relay_endpoint(endpoint))
        }
        AllowedTunnelTraffic::Two(first, second) => {
            winfw::AllowedTunnelTraffic::Two(to_relay_endpoint(first), to_relay_endpoint(second))
        }
    }
}

fn map_init_error(error: winfw::Error, action: &str) -> Error {
    log::error!(
        "{}",
        error.display_chain_with_msg(&format!("Failed to {action} the firewall module"))
    );
    Error::Initialization
}

fn map_apply_error(
    action: &str,
) -> impl Fn(winfw::ApplyError) -> talpid_types::tunnel::FirewallPolicyError {
    move |error| {
        let action = format!("Failed to {action} the firewall");
        match error {
            winfw::ApplyError::Generic(error) => {
                log::error!("{}", error.display_chain_with_msg(&action));
                talpid_types::tunnel::FirewallPolicyError::Generic
            }
            winfw::ApplyError::LockTimeout(error) => {
                log::warn!(
                    "{}",
                    error.display_chain_with_msg(&format!(
                        "{action}: the WFP transaction lock timed out"
                    ))
                );
                talpid_types::tunnel::FirewallPolicyError::Locked(None)
            }
        }
    }
}
