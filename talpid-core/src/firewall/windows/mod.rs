use std::{net::IpAddr, sync::LazyLock};

use talpid_tunnel::TunnelMetadata;
use talpid_types::{
    ErrorExt,
    net::{AllowedEndpoint, AllowedTunnelTraffic},
    tunnel::FirewallPolicyError,
};
use widestring::WideCString;

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, InitialFirewallState};
use talpid_dns::ResolvedDnsConfig;
use windows_sys::core::GUID;

/// Sublayer GUIDs to use for WinFw
const DEFAULT_SUBLAYER_GUIDS: WinFwSublayerGuids = WinFwSublayerGuids {
    baseline: GUID {
        data1: 0xc78056ff,
        data2: 0x2bc1,
        data3: 0x4211,
        data4: [0xaa, 0xdd, 0x7f, 0x35, 0x8d, 0xef, 0x20, 0x2d],
    },
    dns: GUID {
        data1: 0x60090787,
        data2: 0xcca1,
        data3: 0x4937,
        data4: [0xaa, 0xce, 0x51, 0x25, 0x6e, 0xf4, 0x81, 0xf3],
    },
    persistent: GUID {
        data1: 0x3c28881e,
        data2: 0x8891,
        data3: 0x4d61,
        data4: [0xb8, 0x7f, 0xf2, 0x72, 0x50, 0x2d, 0x10, 0x05],
    },
};

/// Fallback sublayer GUIDs used when [`DEFAULT_SUBLAYER_GUIDS`] conflict with other software.
/// The conflict usually occurs because split tunneling depends on the existence of the sublayers,
/// so other VPN clients add them and conflict with this one.
/// If the fallback GUIDs are used, split tunneling must be disabled, as `win-split-tunnel`
/// hardcodes the default GUIDs (except `persistent`).
const FALLBACK_SUBLAYER_GUIDS: WinFwSublayerGuids = WinFwSublayerGuids {
    baseline: GUID {
        data1: 0x6c9d2e4f,
        data2: 0x1a3b,
        data3: 0x5c7d,
        data4: [0x8e, 0x9f, 0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f],
    },
    dns: GUID {
        data1: 0x7d0e3f50,
        data2: 0x2b4c,
        data3: 0x6d8e,
        data4: [0x9f, 0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60],
    },
    persistent: GUID {
        data1: 0x3c28881e,
        data2: 0x8891,
        data3: 0x4d61,
        data4: [0xb8, 0x7f, 0xf2, 0x72, 0x50, 0x2d, 0x10, 0x05],
    },
};

#[macro_use] // must come before other mod declarations
mod ffi;

mod hyperv;
mod winfw;

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

    /// Failure to deinitialize windows firewall module
    #[error("Failed to deinitialize windows firewall module")]
    Deinitialization,

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
    /// If firewall rules should even if firewall module is shut down or dies.
    ///
    /// This should only very cautiously be turned off.
    persist: bool,

    /// Whether split tunneling is available for this session.
    ///
    /// Set to `false` when a WFP sublayer GUID conflict was detected at startup, causing the
    /// firewall to initialize with fallback GUIDs. In that case, `win-split-tunnel` (which
    /// hardcodes the primary GUIDs) must not be used.
    split_tunnel_available: bool,
}

impl Firewall {
    pub fn split_tunnel_available(&self) -> bool {
        self.split_tunnel_available
    }

    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        if let InitialFirewallState::Blocked(allowed_endpoint) = args.initial_state {
            Self::initialize_blocked(allowed_endpoint, args.allow_lan)
        } else {
            Self::new()
        }
    }

    pub fn new() -> Result<Self, Error> {
        let (guids, split_tunnel_available) = Self::guids();
        winfw::initialize(guids)?;
        log::trace!("Successfully initialized windows firewall module");
        Ok(Self {
            split_tunnel_available,
            persist: true,
        })
    }

    fn initialize_blocked(
        allowed_endpoint: AllowedEndpoint,
        allow_lan: bool,
    ) -> Result<Self, Error> {
        let (guids, split_tunnel_available) = Self::guids();
        winfw::initialize_blocked(guids, allowed_endpoint, allow_lan)?;
        log::trace!("Successfully initialized windows firewall module to a blocking state");

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
        });

        Ok(Self {
            split_tunnel_available,
            persist: true,
        })
    }

    fn guids() -> (&'static WinFwSublayerGuids, bool) {
        if winfw::has_sublayer_conflict(&DEFAULT_SUBLAYER_GUIDS) {
            log::warn!("WFP sublayer GUID conflict detected. Disabling split tunneling");
            (&FALLBACK_SUBLAYER_GUIDS, false)
        } else {
            (&DEFAULT_SUBLAYER_GUIDS, true)
        }
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        let should_block_hyperv = matches!(
            policy,
            FirewallPolicy::Connecting { .. } | FirewallPolicy::Blocked { .. }
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
                let cfg = &WinFwSettings::new(allow_lan);
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
                let cfg = &WinFwSettings::new(allow_lan);
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
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(
                    cfg,
                    allowed_endpoint.map(WinFwAllowedEndpointContainer::from),
                )
            }
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
        winfw::reset().map_err(Error::ResettingPolicy)?;

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
        winfw_settings: &WinFwSettings,
        tunnel_metadata: Option<&TunnelMetadata>,
        allowed_endpoint: AllowedEndpoint,
        allowed_tunnel_traffic: &AllowedTunnelTraffic,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connecting' firewall policy");
        let tunnel_interface = tunnel_metadata.map(|metadata| metadata.interface.as_ref());
        winfw::apply_policy_connecting(
            peer_endpoints,
            exit_endpoint_ip,
            winfw_settings,
            tunnel_interface,
            allowed_endpoint,
            allowed_tunnel_traffic,
        )
        .map_err(Error::ApplyingConnectingPolicy)
    }

    fn set_connected_state(
        &mut self,
        peer_endpoints: &[AllowedEndpoint],
        exit_endpoint_ip: Option<IpAddr>,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &TunnelMetadata,
        dns_config: &ResolvedDnsConfig,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connected' firewall policy");
        let tunnel_interface = &tunnel_metadata.interface;
        winfw::apply_policy_connected(
            peer_endpoints,
            exit_endpoint_ip,
            winfw_settings,
            tunnel_interface,
            dns_config,
        )
        .map_err(Error::ApplyingConnectedPolicy)
    }

    fn set_blocked_state(
        &mut self,
        winfw_settings: &WinFwSettings,
        allowed_endpoint: Option<WinFwAllowedEndpointContainer>,
    ) -> Result<(), Error> {
        log::trace!("Applying 'blocked' firewall policy");
        winfw::apply_policy_blocked(winfw_settings, allowed_endpoint)
            .map_err(Error::ApplyingBlockedPolicy)
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        // Deinitialize WinFW with or without persistent filters.
        // All other filters should still remain intact.
        let cleanup_policy = if self.persist {
            WinFwCleanupPolicy::ContinueBlocking
        } else {
            WinFwCleanupPolicy::BlockingUntilReboot
        };

        match winfw::deinit(cleanup_policy) {
            Ok(()) => log::trace!("Successfully deinitialized windows firewall module"),
            Err(_) => log::error!("Failed to deinitialize windows firewall module"),
        }
    }
}

fn widestring_ip(ip: IpAddr) -> WideCString {
    WideCString::from_str_truncate(ip.to_string())
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
