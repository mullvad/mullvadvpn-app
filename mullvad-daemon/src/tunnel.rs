use std::{future::Future, net::IpAddr, pin::Pin, sync::Arc};

use talpid_types::net::wireguard::TunnelParameters;
use tokio::sync::Mutex;

use mullvad_relay_selector::{GetRelay, RelaySelector, WireguardConfig};
use mullvad_types::{
    endpoint::MullvadEndpoint, location::GeoIpLocation, relay_list::Relay, settings::TunnelOptions,
};
use talpid_core::tunnel_state_machine::TunnelParametersGenerator;
use talpid_types::net::{obfuscation::Obfuscators, wireguard};

use talpid_types::{ErrorExt, net::IpAvailability, tunnel::ParameterGenerationError};

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
    relay_selector: RelaySelector,
    tunnel_options: TunnelOptions,
    account_manager: AccountManagerHandle,

    last_generated_relays: Option<LastSelectedRelays>,
}

impl ParametersGenerator {
    /// Constructs a new tunnel parameters generator.
    pub fn new(
        account_manager: AccountManagerHandle,
        relay_selector: RelaySelector,
        tunnel_options: TunnelOptions,
    ) -> Self {
        Self(Arc::new(Mutex::new(InnerParametersGenerator {
            tunnel_options,
            relay_selector,

            account_manager,

            last_generated_relays: None,
        })))
    }

    /// Sets the tunnel options to use when generating new tunnel parameters.
    pub async fn set_tunnel_options(&self, tunnel_options: &TunnelOptions) {
        self.0.lock().await.tunnel_options = tunnel_options.clone();
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

        let take_hostname =
            |relay: &Option<Relay>| relay.as_ref().map(|relay| relay.hostname.clone());

        let entry_hostname = take_hostname(&relays.entry);
        let hostname = relays.exit.hostname.clone();
        let obfuscator_hostname = take_hostname(&relays.obfuscator);
        let location = relays.exit.location.clone();

        Some(GeoIpLocation {
            ipv4: None,
            ipv6: None,
            country: location.country,
            city: Some(location.city),
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: true,
            hostname: Some(hostname),
            entry_hostname,
            obfuscator_hostname,
        })
    }
}

impl InnerParametersGenerator {
    async fn generate(
        &mut self,
        retry_attempt: u32,
        ip_availability: IpAvailability,
    ) -> Result<TunnelParameters, Error> {
        let data = self.device().await?;
        let selected_relay = self
            .relay_selector
            .get_relay(retry_attempt as usize, ip_availability)?;

        match selected_relay {
            GetRelay::Mullvad {
                endpoint,
                obfuscator,
                inner,
            } => {
                let (obfuscator_relay, obfuscator_config) = match obfuscator {
                    Some(obfuscator) => (Some(obfuscator.relay), Some(obfuscator.config)),
                    None => (None, None),
                };

                let (entry, exit) = match inner {
                    WireguardConfig::Singlehop { exit } => (None, exit),
                    WireguardConfig::Multihop { exit, entry } => (Some(entry), exit),
                };
                let server_override = {
                    let first_relay = entry.as_ref().unwrap_or(&exit);
                    (first_relay.overridden_ipv4 && endpoint.peer.endpoint.is_ipv4())
                        || (first_relay.overridden_ipv6 && endpoint.peer.endpoint.is_ipv6())
                };

                self.last_generated_relays = Some(LastSelectedRelays {
                    entry,
                    exit,
                    obfuscator: obfuscator_relay,
                    server_override,
                });

                Ok(self.create_wireguard_tunnel_parameters(endpoint, data, obfuscator_config))
            }
            GetRelay::Custom(custom_relay) => {
                self.last_generated_relays = None;
                custom_relay
                    // TODO: generate proxy settings for custom tunnels
                    .to_tunnel_parameters(self.tunnel_options.clone())
                    .map_err(|e| {
                        log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                        Error::ResolveCustomHostname
                    })
            }
        }
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
    entry: Option<Relay>,
    exit: Relay,
    obfuscator: Option<Relay>,
    server_override: bool,
}
