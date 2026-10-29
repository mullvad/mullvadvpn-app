//! The Windows firewall implementation, backed by the `winfw` crate.
//!
//! The module owns the [`winfw::Manager`] singleton for the process and adapts
//! the daemon's types to it, replacing the previous C FFI to the C++ `winfw`
//! library.

use std::{
    net::IpAddr,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use talpid_types::net::{AllowedTunnelTraffic, Endpoint, TransportProtocol};

// The firewall's types, re-exported for the parent module's path resolution:
// it declares `mod winfw;`, so paths like `winfw::Settings` resolve to this
// adapter module rather than the external crate.
pub use winfw::Settings;

use talpid_types::net::AllowedEndpoint;

// Kept from the previous FFI layer for readable path resolution in the parent
// module: it declares `mod winfw;`, so paths like `winfw::Settings` resolve to
// this adapter module rather than the external crate.

/// Timeout for acquiring the WFP transaction lock.
const TRANSACTION_TIMEOUT: Duration = Duration::from_secs(5);

/// The firewall singleton. The daemon initializes it exactly once per process,
/// exactly like the C library it replaces did.
static WINFW: LazyLock<Mutex<Option<winfw::Manager>>> = LazyLock::new(|| Mutex::new(None));

/// Errors when initializing the module.
#[derive(thiserror::Error, Debug)]
#[error("WinFW manager state error")]
pub struct StateError;

fn lock_manager() -> Result<std::sync::MutexGuard<'static, Option<winfw::Manager>>, StateError> {
    WINFW.lock().map_err(|_| StateError)
}

/// The policy currently in effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePolicy {
    None,
    Connecting,
    Connected,
    Blocked,
}

/// How to clean up when deinitializing the module. The daemon only ever
/// leaves the blocking filters in place, expecting the persistent objects to
/// take over for them.
#[expect(dead_code)]
pub enum CleanupPolicy {
    /// Remove all objects that have been registered with WFP.
    ResetFirewall,
    /// Continue blocking if this is the active policy. The blocking filters
    /// are ephemeral: they are active until the machine is rebooted.
    BlockingUntilReboot,
}

/// Initialize the firewall module.
pub(super) fn initialize() -> Result<(), super::Error> {
    let mut manager = lock_manager().map_err(|_| super::Error::Initialization)?;
    if manager.is_some() {
        return Err(super::Error::Initialization);
    }
    *manager = Some(
        winfw::Manager::new(TRANSACTION_TIMEOUT)
            .map_err(|error| map_init_error(error, "initialize"))?,
    );
    Ok(())
}

/// Initialize the firewall module and apply blocking rules.
pub(super) fn initialize_blocked(
    allowed_endpoint: AllowedEndpoint,
    allow_lan: bool,
) -> Result<(), super::Error> {
    let mut manager = lock_manager().map_err(|_| super::Error::Initialization)?;
    if manager.is_some() {
        return Err(super::Error::Initialization);
    }

    let endpoint = to_allowed_endpoint(&allowed_endpoint);
    *manager = Some(
        winfw::Manager::new_blocked(
            TRANSACTION_TIMEOUT,
            &winfw::Settings::new(allow_lan),
            Some(&endpoint),
        )
        .map_err(|error| map_init_error(error, "initialize to a blocking state"))?,
    );
    Ok(())
}

/// Deinitialize the module.
pub(super) fn deinit(cleanup: CleanupPolicy) -> Result<(), super::Error> {
    let mut manager = lock_manager().map_err(|_| super::Error::Deinitialization)?;
    let Some(mut instance) = manager.take() else {
        // Deinitializing an uninitialized module is a no-op, like in the C
        // library.
        return Ok(());
    };

    let cleanup = match cleanup {
        CleanupPolicy::ResetFirewall => winfw::CleanupPolicy::ResetFirewall,
        CleanupPolicy::BlockingUntilReboot => winfw::CleanupPolicy::BlockingUntilReboot,
    };
    instance
        .deinitialize(cleanup)
        .map_err(|_| super::Error::Deinitialization)
}

/// The policy currently in effect, or [`ActivePolicy::None`] if the module is
/// not initialized.
pub(super) fn active_policy() -> ActivePolicy {
    let Ok(manager) = lock_manager() else {
        return ActivePolicy::None;
    };
    match manager.as_ref().map(winfw::Manager::active_policy) {
        Some(winfw::ActivePolicy::None) | None => ActivePolicy::None,
        Some(winfw::ActivePolicy::Connecting) => ActivePolicy::Connecting,
        Some(winfw::ActivePolicy::Connected) => ActivePolicy::Connected,
        Some(winfw::ActivePolicy::Blocked) => ActivePolicy::Blocked,
    }
}

/// Clear the policy in effect, if any.
pub(super) fn reset() -> Result<(), talpid_types::tunnel::FirewallPolicyError> {
    let mut manager =
        lock_manager().map_err(|_| talpid_types::tunnel::FirewallPolicyError::Generic)?;
    match manager.as_mut() {
        Some(instance) => instance
            .reset()
            .map_err(map_apply_error("reset the policy of")),
        None => winfw::purge_all().map_err(map_apply_error("remove all objects of")),
    }
}

/// Apply the connecting policy.
pub(super) fn apply_policy_connecting(
    peer_endpoints: &[AllowedEndpoint],
    exit_endpoint_ip: Option<IpAddr>,
    winfw_settings: &winfw::Settings,
    tunnel_interface: Option<&str>,
    allowed_endpoint: AllowedEndpoint,
    allowed_tunnel_traffic: &AllowedTunnelTraffic,
) -> Result<(), talpid_types::tunnel::FirewallPolicyError> {
    let mut manager =
        lock_manager().map_err(|_| talpid_types::tunnel::FirewallPolicyError::Generic)?;
    let Some(instance) = manager.as_mut() else {
        log::error!("Cannot apply connecting policy: WinFW not initialized");
        return Err(talpid_types::tunnel::FirewallPolicyError::Generic);
    };

    let (relays, tunnel_clients) = to_relays(peer_endpoints)?;

    let tunnel = winfw::TunnelParameters {
        alias: tunnel_interface.map(str::to_owned),
        exit_ip: exit_endpoint_ip,
        clients: tunnel_clients,
        allowed_tunnel_traffic: to_allowed_tunnel_traffic(allowed_tunnel_traffic),
    };

    instance
        .apply_policy_connecting(
            winfw_settings,
            &relays,
            &tunnel,
            &to_allowed_endpoint(&allowed_endpoint),
        )
        .map_err(map_apply_error("apply the connecting policy of"))
}

/// Apply the connected policy.
pub(super) fn apply_policy_connected(
    peer_endpoints: &[AllowedEndpoint],
    exit_endpoint_ip: Option<IpAddr>,
    winfw_settings: &winfw::Settings,
    tunnel_interface: &str,
    dns_config: &talpid_dns::ResolvedDnsConfig,
) -> Result<(), talpid_types::tunnel::FirewallPolicyError> {
    let mut manager =
        lock_manager().map_err(|_| talpid_types::tunnel::FirewallPolicyError::Generic)?;
    let Some(instance) = manager.as_mut() else {
        log::error!("Cannot apply connected policy: WinFW not initialized");
        return Err(talpid_types::tunnel::FirewallPolicyError::Generic);
    };

    let (relays, tunnel_clients) = to_relays(peer_endpoints)?;

    let tunnel = winfw::TunnelParameters {
        alias: Some(tunnel_interface.to_owned()),
        exit_ip: exit_endpoint_ip,
        clients: tunnel_clients,
        allowed_tunnel_traffic: winfw::AllowedTunnelTraffic::All,
    };

    instance
        .apply_policy_connected(
            winfw_settings,
            &relays,
            &tunnel,
            dns_config.tunnel_config(),
            dns_config.non_tunnel_config(),
        )
        .map_err(map_apply_error("apply the connected policy of"))
}

/// Apply the blocked policy.
pub(super) fn apply_policy_blocked(
    winfw_settings: &winfw::Settings,
    allowed_endpoint: Option<&winfw::AllowedEndpoint>,
) -> Result<(), talpid_types::tunnel::FirewallPolicyError> {
    let mut manager =
        lock_manager().map_err(|_| talpid_types::tunnel::FirewallPolicyError::Generic)?;
    let Some(instance) = manager.as_mut() else {
        log::error!("Cannot apply blocked policy: WinFW not initialized");
        return Err(talpid_types::tunnel::FirewallPolicyError::Generic);
    };

    instance
        .apply_policy_blocked(winfw_settings, allowed_endpoint)
        .map_err(map_apply_error("apply the blocked policy of"))
}

/// The endpoint allowed outside the tunnel, in the `winfw` crate's form.
pub(super) fn to_allowed_endpoint(endpoint: &AllowedEndpoint) -> winfw::AllowedEndpoint {
    winfw::AllowedEndpoint {
        clients: endpoint
            .clients
            .iter()
            .map(|client| client.display().to_string())
            .collect(),
        endpoint: winfw::RelayEndpoint {
            ip: endpoint.endpoint.address.ip(),
            port: endpoint.endpoint.address.port(),
            protocol: to_protocol(endpoint.endpoint.protocol),
        },
    }
}

/// Convert the peer endpoints into relays, requiring they allow the same
/// clients, like the previous FFI layer did.
fn to_relays(
    peer_endpoints: &[AllowedEndpoint],
) -> Result<(Vec<winfw::RelayEndpoint>, Vec<String>), talpid_types::tunnel::FirewallPolicyError> {
    let mut relays = Vec::with_capacity(peer_endpoints.len());
    let mut tunnel_clients: Option<&[std::path::PathBuf]> = None;

    for peer in peer_endpoints {
        if let Some(clients) = tunnel_clients {
            if clients != peer.clients.as_ref() {
                log::error!("Relay endpoints must have same allowed clients");
                return Err(talpid_types::tunnel::FirewallPolicyError::Generic);
            }
        } else {
            tunnel_clients = Some(&peer.clients);
        }

        relays.push(winfw::RelayEndpoint {
            ip: peer.endpoint.address.ip(),
            port: peer.endpoint.address.port(),
            protocol: to_protocol(peer.endpoint.protocol),
        });
    }

    let clients = tunnel_clients
        .ok_or_else(|| {
            log::error!("Allowed clients missing. Zero peer endpoints?");
            talpid_types::tunnel::FirewallPolicyError::Generic
        })?
        .iter()
        .map(|client| client.display().to_string())
        .collect();

    Ok((relays, clients))
}

fn to_protocol(protocol: TransportProtocol) -> winfw::Protocol {
    match protocol {
        TransportProtocol::Tcp => winfw::Protocol::Tcp,
        TransportProtocol::Udp => winfw::Protocol::Udp,
    }
}

fn to_allowed_tunnel_traffic(traffic: &AllowedTunnelTraffic) -> winfw::AllowedTunnelTraffic {
    let to_tunnel_endpoint = |endpoint: &Endpoint| winfw::RelayEndpoint {
        ip: endpoint.address.ip(),
        port: endpoint.address.port(),
        protocol: to_protocol(endpoint.protocol),
    };
    match traffic {
        AllowedTunnelTraffic::None => winfw::AllowedTunnelTraffic::None,
        AllowedTunnelTraffic::All => winfw::AllowedTunnelTraffic::All,
        AllowedTunnelTraffic::One(endpoint) => {
            winfw::AllowedTunnelTraffic::One(to_tunnel_endpoint(endpoint))
        }
        AllowedTunnelTraffic::Two(first, second) => {
            winfw::AllowedTunnelTraffic::Two(to_tunnel_endpoint(first), to_tunnel_endpoint(second))
        }
    }
}

fn map_init_error(error: winfw::Error, action: &str) -> super::Error {
    use talpid_error::ErrorExt as _;

    log::error!(
        "{}",
        error.display_chain_with_msg(&format!("Failed to {action} the firewall module"))
    );
    super::Error::Initialization
}

fn map_apply_error(
    action: &str,
) -> impl Fn(winfw::ApplyError) -> talpid_types::tunnel::FirewallPolicyError {
    use talpid_error::ErrorExt as _;

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
