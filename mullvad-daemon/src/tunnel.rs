use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::{future::Future, net::IpAddr, pin::Pin, sync::Arc};
use talpid_error::ErrorExt;

use talpid_types::net::wireguard::TunnelParameters;
use tokio::sync::Mutex;

use mullvad_daemon_relay_selector::relay_selector::RelaySelectorIO;
use mullvad_relay_selector::{GetRelay, WireguardConfig};
use mullvad_types::{
    endpoint::MullvadEndpoint,
    location::GeoIpLocation,
    relay_constraints::RelaySettings,
    settings::{Settings, TunnelOptions},
};
use talpid_core::tunnel_state_machine::TunnelParametersGenerator;
use talpid_types::net::{
    ALLOWED_LAN_MULTICAST_NETS, ALLOWED_LAN_NETS, ipnetwork_sub::IpNetworkSub,
    obfuscation::Obfuscators, wireguard,
};
use talpid_types::{net::IpAvailability, tunnel::ParameterGenerationError};

use crate::device::{AccountManagerHandle, Error as DeviceError, PrivateAccountAndDevice};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not logged in on a valid device")]
    NoAuthDetails,

    #[error("Failed to select a matching relay")]
    SelectRelay(#[from] mullvad_relay_selector::Error),

    #[error("Failed to resolve hostname for custom relay")]
    ResolveCustomHostname,

    #[error("Failed to get device data")]
    Device(#[from] DeviceError),
}

#[derive(Clone)]
pub(crate) struct ParametersGenerator(Arc<Mutex<InnerParametersGenerator>>);

struct InnerParametersGenerator {
    relay_selector: RelaySelectorIO,
    relay_settings: RelaySettings,
    tunnel_options: TunnelOptions,
    account_manager: AccountManagerHandle,

    last_generated_relays: Option<LastSelectedRelays>,
}

impl ParametersGenerator {
    /// Constructs a new tunnel parameters generator.
    pub fn new(
        account_manager: AccountManagerHandle,
        relay_selector: RelaySelectorIO,
        relay_settings: RelaySettings,
        tunnel_options: TunnelOptions,
    ) -> Self {
        Self(Arc::new(Mutex::new(InnerParametersGenerator {
            tunnel_options,
            relay_selector,
            relay_settings,
            account_manager,
            last_generated_relays: None,
        })))
    }

    /// Sets the tunnel options to use when generating new tunnel parameters.
    pub async fn set_tunnel_options(&self, tunnel_options: &TunnelOptions) {
        self.0.lock().await.tunnel_options = tunnel_options.clone();
    }

    /// Updates generator state from full settings and keeps relay-selector config in sync.
    pub async fn set_settings(&self, settings: Settings) {
        let mut inner = self.0.lock().await;
        inner.relay_settings = settings.relay_settings.clone();
        inner.relay_selector.set_config(settings);
    }

    pub async fn last_relay_was_overridden(&self) -> bool {
        let inner = self.0.lock().await;
        let Some(relays) = inner.last_generated_relays.as_ref() else {
            return false;
        };
        relays.server_override
    }

    /// Gets the location associated with the last generated tunnel parameters.
    pub async fn get_last_location(&self) -> Option<GeoIpLocation> {
        let inner = self.0.lock().await;

        let relays = inner.last_generated_relays.as_ref()?;

        let (entry, exit) = match &relays.config {
            WireguardConfig::Singlehop { exit } => (None, exit),
            WireguardConfig::Multihop { exit, entry } => (Some(entry), exit),
        };
        let location = exit.location.clone();

        Some(GeoIpLocation {
            latitude: location.latitude,
            longitude: location.longitude,
            ipv4: None,
            ipv6: None,
            mullvad_exit_ip: true,
            hostname: Some(exit.hostname.clone()),
            city: Some(location.city),
            country: location.country,
            entry_hostname: entry.map(|relay| relay.hostname.clone()),
            entry_city: entry.map(|relay| relay.location.city.clone()),
            entry_country: entry.map(|relay| relay.location.country.clone()),
        })
    }
}

/// Private networks that may point to Mullvad services in the VPN tunnel.
const ALLOWED_IN_TUNNEL_LAN_NETS: [IpNetwork; 2] = [
    // Net including the relay IPv4 gateway. Used for DNS, tunnel config service, and connectivity
    // check.
    // This also includes SOCKS5 proxies.
    // Reserve all of `10/8` in case new services are added.
    IpNetwork::V4(Ipv4Network::new_checked(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
    // Net including the relay IPv6 gateway.
    IpNetwork::V6(
        Ipv6Network::new_checked(
            Ipv6Addr::new(0xfc00, 0xbbbb, 0xbbbb, 0xbb01, 0, 0, 0, 0),
            64,
        )
        .unwrap(),
    ),
];

/// Remove the private networks that must not be reachable through a Mullvad tunnel from
/// `allowed_ips`.
///
/// [`ALLOWED_IN_TUNNEL_LAN_NETS`] is allowlisted, since those ranges are legitimately used in
/// Mullvad tunnels. Other traffic is likely just LAN traffic leaking into the tunnel.
fn subtract_lan_nets(allowed_ips: &[IpNetwork]) -> Vec<IpNetwork> {
    // `sub_all` cannot mix address families.
    let blocked = |ipv4: bool| -> Vec<IpNetwork> {
        let exceptions: Vec<IpNetwork> = ALLOWED_IN_TUNNEL_LAN_NETS
            .iter()
            .copied()
            .filter(|net| net.is_ipv4() == ipv4)
            .collect();

        ALLOWED_LAN_NETS
            .iter()
            .chain(ALLOWED_LAN_MULTICAST_NETS.iter())
            .filter(|net| net.is_ipv4() == ipv4)
            .flat_map(|lan_net| lan_net.sub_all(exceptions.clone()))
            .collect()
    };
    let (blocked_v4, blocked_v6) = (blocked(true), blocked(false));

    allowed_ips
        .iter()
        .flat_map(|allowed_ip| {
            let blocked = if allowed_ip.is_ipv4() {
                &blocked_v4
            } else {
                &blocked_v6
            };
            allowed_ip.sub_all(blocked.clone())
        })
        .collect()
}

impl InnerParametersGenerator {
    async fn generate(
        &mut self,
        retry_attempt: u32,
        ip_availability: IpAvailability,
    ) -> Result<TunnelParameters, Error> {
        // Custom tunnel endpoints bypass relay selection entirely.
        if let RelaySettings::CustomTunnelEndpoint(ref endpoint) = self.relay_settings {
            self.last_generated_relays = None;
            return endpoint
                .to_tunnel_parameters(self.tunnel_options.clone())
                .map_err(|e| {
                    log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                    Error::ResolveCustomHostname
                });
        }

        let data = self.device().await?;
        let selected_relay = self
            .relay_selector
            .get_relay(retry_attempt as usize, ip_availability)?;

        let GetRelay {
            endpoint,
            obfuscator,
            inner,
        } = selected_relay;

        let server_override = {
            let first_relay = match &inner {
                WireguardConfig::Singlehop { exit } => exit,
                WireguardConfig::Multihop { exit: _, entry } => entry,
            };
            match endpoint.peer.endpoint {
                SocketAddr::V4(_) => first_relay.overridden_ipv4,
                SocketAddr::V6(_) => first_relay.overridden_ipv6,
            }
        };

        self.last_generated_relays = Some(LastSelectedRelays {
            config: inner,
            server_override,
        });

        Ok(self.create_wireguard_tunnel_parameters(endpoint, data, obfuscator))
    }

    fn create_wireguard_tunnel_parameters(
        &self,
        mut endpoint: MullvadEndpoint,
        data: PrivateAccountAndDevice,
        obfuscator_config: Option<Obfuscators>,
    ) -> TunnelParameters {
        let tunnel_ipv4 = data.device.wg_data.addresses.ipv4_address.ip();
        let tunnel_ipv6 = data.device.wg_data.addresses.ipv6_address.ip();
        let tunnel = wireguard::TunnelConfig {
            private_key: data.device.wg_data.private_key,
            addresses: vec![IpAddr::from(tunnel_ipv4), IpAddr::from(tunnel_ipv6)],
        };

        // Deliberately filter out private IP ranges but keep the routes, to prevent unexpected
        // LAN traffic in the tunnel.
        let routes = Some(
            endpoint
                .exit_peer
                .iter()
                .chain(std::iter::once(&endpoint.peer))
                .flat_map(|peer| peer.allowed_ips.iter())
                .copied()
                .collect(),
        );

        for peer in endpoint
            .exit_peer
            .iter_mut()
            .chain(std::iter::once(&mut endpoint.peer))
        {
            peer.allowed_ips = subtract_lan_nets(&peer.allowed_ips);
        }

        wireguard::TunnelParameters {
            connection: wireguard::ConnectionConfig {
                tunnel,
                peer: endpoint.peer,
                exit_peer: endpoint.exit_peer,
                ipv4_gateway: endpoint.ipv4_gateway,
                ipv6_gateway: Some(endpoint.ipv6_gateway),
                routes,
                #[cfg(target_os = "linux")]
                fwmark: Some(mullvad_types::TUNNEL_FWMARK),
            },
            options: self
                .tunnel_options
                .wireguard
                .clone()
                .into_talpid_tunnel_options(),
            generic_options: self.tunnel_options.generic.clone(),
            obfuscation: obfuscator_config,
        }
    }

    async fn device(&self) -> Result<PrivateAccountAndDevice, Error> {
        let device_state = self.account_manager.data().await?;
        device_state.into_device().ok_or(Error::NoAuthDetails)
    }
}

impl TunnelParametersGenerator for ParametersGenerator {
    fn generate(
        &mut self,
        retry_attempt: u32,
        ip_availability: IpAvailability,
    ) -> Pin<Box<dyn Future<Output = Result<TunnelParameters, ParameterGenerationError>>>> {
        let generator = self.0.clone();
        Box::pin(async move {
            let mut inner = generator.lock().await;
            inner
                .generate(retry_attempt, ip_availability)
                .await
                .inspect_err(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to generate tunnel parameters")
                    );
                })
                .map_err(ParameterGenerationError::from)
        })
    }
}

impl From<Error> for ParameterGenerationError {
    fn from(error: Error) -> Self {
        match error {
            Error::SelectRelay(mullvad_relay_selector::Error::NoBridge) => {
                ParameterGenerationError::NoMatchingBridgeRelay
            }
            Error::ResolveCustomHostname => {
                ParameterGenerationError::CustomTunnelHostResolutionError
            }
            Error::SelectRelay(mullvad_relay_selector::Error::IpVersionUnavailable { family }) => {
                ParameterGenerationError::IpVersionUnavailable { family }
            }
            Error::SelectRelay(mullvad_relay_selector::Error::NoRelayEntry(_)) => {
                ParameterGenerationError::NoMatchingRelayEntry
            }
            Error::SelectRelay(mullvad_relay_selector::Error::NoRelayExit(_)) => {
                ParameterGenerationError::NoMatchingRelayExit
            }
            Error::NoAuthDetails | Error::SelectRelay(_) | Error::Device(_) => {
                ParameterGenerationError::NoMatchingRelay
            }
        }
    }
}

/// Contains all relays that were selected last time when tunnel parameters were generated.
///
/// Represents all relays generated for a WireGuard tunnel.
/// The traffic flow can look like this:
///     client -> obfuscator -> entry -> exit -> internet
/// But for most users, it will look like this:
///     client -> entry -> internet
struct LastSelectedRelays {
    config: WireguardConfig,
    server_override: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    fn allowed_ips(nets: &[&str]) -> Vec<IpNetwork> {
        nets.iter().map(|net| net.parse().unwrap()).collect()
    }

    fn covers(nets: &[IpNetwork], ip: &str) -> bool {
        let ip: IpAddr = ip.parse().unwrap();
        nets.iter().any(|net| net.contains(ip))
    }

    /// Test whether private IPs are subtracted correctly.
    #[test]
    fn test_disallowed_tun_ip_ranges() {
        let result = subtract_lan_nets(&allowed_ips(&["0.0.0.0/0", "::/0"]));

        for ip in ["192.168.1.1", "172.16.0.1", "169.254.0.1", "fe80::1"] {
            assert!(!covers(&result, ip), "{ip} must not be reachable in tunnel");
        }
        for ip in ["1.2.3.4", "2606:4700::1111"] {
            assert!(covers(&result, ip), "{ip} must remain reachable in tunnel");
        }
    }

    /// Test legit private IPs in Mullvad tunnels.
    ///
    /// - IPv4 gateway/DNS: 10.64.0.1
    /// - SOCKS proxies: 10.124.0.0/23
    /// - Possible future range: 10.128.0.1
    /// - Adblocking DNS: 100.64.0.1
    #[test]
    fn test_allowed_mullvad_tun_ip_ranges() {
        let result = subtract_lan_nets(&allowed_ips(&["0.0.0.0/0", "::/0"]));

        for ip in ["10.64.0.1", "10.124.0.2", "10.128.0.1", "100.64.0.1"] {
            assert!(covers(&result, ip), "{ip} must be reachable in tunnel");
        }
    }
}
