use std::net::SocketAddr;
use std::{future::Future, net::IpAddr, pin::Pin, sync::Arc};

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
    Endpoint, TransportProtocol,
    obfuscation::Obfuscators,
    proxy::{Socks5Local, Socks5Proxy, Socks5Remote},
    wireguard,
};
use talpid_types::{ErrorExt, net::IpAvailability, tunnel::ParameterGenerationError};

use crate::device::{AccountManagerHandle, Error as DeviceError, PrivateAccountAndDevice};

/// Relay the tunnel through a SOCKS5 proxy. Intended for development, until the proxy can be
/// configured through the settings.
///
/// Accepts either `<proxy address>` for a remote proxy, or `<local port>@<upstream address>` for
/// a proxy running on localhost, where the upstream address is whatever the proxy itself forwards
/// to.
const SOCKS5_PROXY_VAR: &str = "MULLVAD_SOCKS5_PROXY";

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
        endpoint: MullvadEndpoint,
        data: PrivateAccountAndDevice,
        obfuscator_config: Option<Obfuscators>,
    ) -> TunnelParameters {
        let tunnel_ipv4 = data.device.wg_data.addresses.ipv4_address.ip();
        let tunnel_ipv6 = data.device.wg_data.addresses.ipv6_address.ip();
        let tunnel = wireguard::TunnelConfig {
            private_key: data.device.wg_data.private_key,
            addresses: vec![IpAddr::from(tunnel_ipv4), IpAddr::from(tunnel_ipv6)],
        };

        wireguard::TunnelParameters {
            connection: wireguard::ConnectionConfig {
                tunnel,
                peer: endpoint.peer,
                exit_peer: endpoint.exit_peer,
                ipv4_gateway: endpoint.ipv4_gateway,
                ipv6_gateway: Some(endpoint.ipv6_gateway),
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
            proxy: socks5_proxy_override(),
        }
    }

    async fn device(&self) -> Result<PrivateAccountAndDevice, Error> {
        let device_state = self.account_manager.data().await?;
        device_state.into_device().ok_or(Error::NoAuthDetails)
    }
}

/// Parse [`SOCKS5_PROXY_VAR`], if set.
///
/// An unparseable value is logged and ignored rather than failing the connection, since this is a
/// development aid.
fn socks5_proxy_override() -> Option<Socks5Proxy> {
    let value = std::env::var(SOCKS5_PROXY_VAR).ok()?;

    let proxy = match value.split_once('@') {
        Some((local_port, upstream)) => {
            local_port
                .parse()
                .ok()
                .zip(upstream.parse().ok())
                .map(|(local_port, upstream)| {
                    // TODO: The upstream protocol is not always TCP. A local proxy speaking
                    // VLESS over TLS or REALITY uses TCP, but one speaking QUIC, mKCP or
                    // Hysteria uses UDP, and the firewall matches the protocol along with the
                    // address. It cannot be derived from SOCKS5, so it has to come from the
                    // configuration -- along with the rest of what the local case needs, since
                    // `get_next_hop_endpoints` cannot express `AllowedClients::all()` either.
                    Socks5Proxy::Local(Socks5Local {
                        remote_endpoint: Endpoint::from_socket_address(
                            upstream,
                            TransportProtocol::Tcp,
                        ),
                        local_port,
                    })
                })
        }
        None => value.parse().ok().map(|endpoint| {
            Socks5Proxy::Remote(Socks5Remote {
                endpoint,
                auth: None,
            })
        }),
    };

    if proxy.is_none() {
        log::error!("Ignoring malformed {SOCKS5_PROXY_VAR}: {value}");
    }
    proxy
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
