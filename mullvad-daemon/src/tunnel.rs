use std::sync::{Arc, Mutex};

use futures::{channel::mpsc, StreamExt};
use mullvad_relay_selector::{RelaySelector, SelectedBridge, SelectedObfuscator, SelectedRelay};
use mullvad_types::{
    endpoint::MullvadEndpoint, location::GeoIpLocation, relay_list::Relay, settings::TunnelOptions,
    wireguard::WireguardData,
};
use talpid_core::tunnel_state_machine::TunnelParametersGenerator;
use talpid_types::{
    net::{wireguard, TunnelParameters},
    tunnel::ParameterGenerationError,
    ErrorExt,
};

#[cfg(not(target_os = "android"))]
use talpid_types::net::openvpn;

use crate::device::AccountManagerHandle;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Not logged in on a valid device")]
    NoAuthDetails,

    #[error(display = "No bridge available")]
    NoRelayAvailable,

    #[error(display = "No bridge available")]
    NoBridgeAvailable,

    #[error(display = "Failed to resolve hostname for custom relay")]
    ResolveCustomHostnameError,
}

#[derive(Clone)]
pub(crate) struct ParametersGenerator(Arc<Mutex<InnerParametersGenerator>>);

struct InnerParametersGenerator {
    relay_selector: RelaySelector,
    tunnel_options: TunnelOptions,
    auth_details: Option<AuthDetails>,

    // TODO: Move this to `RelaySelector`?
    last_generated_relays: Option<LastSelectedRelays>,
}

struct AuthDetails {
    #[cfg(not(target_os = "android"))]
    openvpn_user: String,
    wg_data: WireguardData,
}

impl ParametersGenerator {
    /// Constructs a new tunnel parameters generator.
    pub async fn new(
        account_manager: AccountManagerHandle,
        relay_selector: RelaySelector,
        tunnel_options: TunnelOptions,
    ) -> Result<Self, Error> {
        let self_ = Self(Arc::new(Mutex::new(InnerParametersGenerator {
            tunnel_options,
            relay_selector,

            auth_details: None,

            last_generated_relays: None,
        })));

        self_.spawn_auth_updater(account_manager).await?;

        Ok(self_)
    }

    /// Sets the tunnel options to use when generating new tunnel parameters.
    pub fn set_tunnel_options(&self, tunnel_options: &TunnelOptions) {
        self.0.lock().unwrap().tunnel_options = tunnel_options.clone();
    }

    /// Gets the location associated with the last generated tunnel parameters.
    pub fn get_last_location(&self) -> Option<GeoIpLocation> {
        let inner = self.0.lock().unwrap();

        let relays = inner.last_generated_relays.as_ref()?;

        let hostname;
        let bridge_hostname;
        let entry_hostname;
        let obfuscator_hostname;
        let location;
        let take_hostname =
            |relay: &Option<Relay>| relay.as_ref().map(|relay| relay.hostname.clone());

        match relays {
            LastSelectedRelays::WireGuard {
                wg_entry: entry,
                wg_exit: exit,
                obfuscator,
            } => {
                entry_hostname = take_hostname(entry);
                hostname = exit.hostname.clone();
                obfuscator_hostname = take_hostname(obfuscator);
                bridge_hostname = None;
                location = exit.location.as_ref().cloned().unwrap();
            }
            #[cfg(not(target_os = "android"))]
            LastSelectedRelays::OpenVpn { relay, bridge } => {
                hostname = relay.hostname.clone();
                bridge_hostname = take_hostname(bridge);
                entry_hostname = None;
                obfuscator_hostname = None;
                location = relay.location.as_ref().cloned().unwrap();
            }
        };

        Some(GeoIpLocation {
            ipv4: None,
            ipv6: None,
            country: location.country,
            city: Some(location.city),
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: true,
            hostname: Some(hostname),
            bridge_hostname,
            entry_hostname,
            obfuscator_hostname,
        })
    }

    async fn spawn_auth_updater(&self, account_manager: AccountManagerHandle) -> Result<(), Error> {
        let (device_event_tx, mut device_event_rx) = mpsc::unbounded();

        let account_manager_copy = account_manager.clone();

        let self_ = Arc::downgrade(&self.0);

        tokio::spawn(async move {
            account_manager
                .receive_events(device_event_tx)
                .await
                .expect("failed to register event listener");

            while let Some(event) = device_event_rx.next().await {
                match self_.upgrade() {
                    Some(self_) => {
                        self_.lock().unwrap().auth_details =
                            event.into_data().map(|data| AuthDetails {
                                #[cfg(not(target_os = "android"))]
                                openvpn_user: data.token,
                                wg_data: data.wg_data,
                            });
                    }
                    None => return,
                }
            }
        });

        self.0.lock().unwrap().auth_details = account_manager_copy
            .data()
            .await
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to initialize device")
                );
                Error::NoAuthDetails
            })?
            .map(|data| AuthDetails {
                #[cfg(not(target_os = "android"))]
                openvpn_user: data.token,
                wg_data: data.wg_data,
            });

        Ok(())
    }
}

impl InnerParametersGenerator {
    fn generate(&mut self, retry_attempt: u32) -> Result<TunnelParameters, Error> {
        let _auth_details = self.auth_details.as_ref().ok_or(Error::NoAuthDetails)?;

        match self.relay_selector.get_relay(retry_attempt) {
            Ok((SelectedRelay::Custom(custom_relay), _bridge, _obfsucator)) => {
                custom_relay
                    // TODO: generate proxy settings for custom tunnels
                    .to_tunnel_parameters(self.tunnel_options.clone(), None)
                    .map_err(|e| {
                        log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                        Error::ResolveCustomHostnameError
                    })
            }
            Ok((SelectedRelay::Normal(constraints), bridge, obfuscator)) => self
                .create_tunnel_parameters(
                    &constraints.exit_relay,
                    &constraints.entry_relay,
                    constraints.endpoint,
                    bridge,
                    obfuscator,
                ),
            Err(mullvad_relay_selector::Error::NoBridge) => Err(Error::NoBridgeAvailable),
            Err(_error) => Err(Error::NoRelayAvailable),
        }
    }

    #[cfg_attr(target_os = "android", allow(unused_variables))]
    fn create_tunnel_parameters(
        &mut self,
        relay: &Relay,
        entry_relay: &Option<Relay>,
        endpoint: MullvadEndpoint,
        bridge: Option<SelectedBridge>,
        obfuscator: Option<SelectedObfuscator>,
    ) -> Result<TunnelParameters, Error> {
        let auth_details = self.auth_details.as_ref().ok_or(Error::NoAuthDetails)?;

        match endpoint {
            #[cfg(not(target_os = "android"))]
            MullvadEndpoint::OpenVpn(endpoint) => {
                let (bridge_settings, bridge_relay) = match bridge {
                    Some(SelectedBridge::Normal(bridge)) => {
                        (Some(bridge.settings), Some(bridge.relay))
                    }
                    Some(SelectedBridge::Custom(settings)) => (Some(settings), None),
                    None => (None, None),
                };

                self.last_generated_relays = Some(LastSelectedRelays::OpenVpn {
                    relay: relay.clone(),
                    bridge: bridge_relay,
                });

                Ok(openvpn::TunnelParameters {
                    config: openvpn::ConnectionConfig::new(
                        endpoint,
                        auth_details.openvpn_user.clone(),
                        "-".to_string(),
                    ),
                    options: self.tunnel_options.openvpn.clone(),
                    generic_options: self.tunnel_options.generic.clone(),
                    proxy: bridge_settings,
                }
                .into())
            }
            #[cfg(target_os = "android")]
            MullvadEndpoint::OpenVpn(endpoint) => {
                unreachable!("OpenVPN is not supported on Android");
            }
            MullvadEndpoint::Wireguard(endpoint) => {
                let tunnel = wireguard::TunnelConfig {
                    private_key: auth_details.wg_data.private_key.clone(),
                    addresses: vec![
                        auth_details.wg_data.addresses.ipv4_address.ip().into(),
                        auth_details.wg_data.addresses.ipv6_address.ip().into(),
                    ],
                };

                let (obfuscator_relay, obfuscator_config) = match obfuscator {
                    Some(obfuscator) => (Some(obfuscator.relay), Some(obfuscator.config)),
                    None => (None, None),
                };

                self.last_generated_relays = Some(LastSelectedRelays::WireGuard {
                    wg_entry: entry_relay.clone(),
                    wg_exit: relay.clone(),
                    obfuscator: obfuscator_relay,
                });

                Ok(wireguard::TunnelParameters {
                    connection: wireguard::ConnectionConfig {
                        tunnel,
                        peer: endpoint.peer,
                        exit_peer: endpoint.exit_peer,
                        ipv4_gateway: endpoint.ipv4_gateway,
                        ipv6_gateway: Some(endpoint.ipv6_gateway),
                    },
                    options: self.tunnel_options.wireguard.options.clone(),
                    generic_options: self.tunnel_options.generic.clone(),
                    obfuscation: obfuscator_config,
                }
                .into())
            }
        }
    }
}

impl TunnelParametersGenerator for ParametersGenerator {
    fn generate(
        &mut self,
        retry_attempt: u32,
    ) -> Result<TunnelParameters, ParameterGenerationError> {
        let mut inner = self.0.lock().unwrap();
        inner.generate(retry_attempt).map_err(|error| match error {
            Error::NoBridgeAvailable => ParameterGenerationError::NoMatchingBridgeRelay,
            Error::ResolveCustomHostnameError => {
                ParameterGenerationError::CustomTunnelHostResultionError
            }
            error => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to generate tunnel parameters")
                );
                ParameterGenerationError::NoMatchingRelay
            }
        })
    }
}

/// Contains all relays that were selected last time when tunnel parameters were generated.
enum LastSelectedRelays {
    /// Represents all relays generated for a WireGuard tunnel.
    /// The traffic flow can look like this:
    ///     client -> obfuscator -> entry -> exit -> internet
    /// But for most users, it will look like this:
    ///     client -> entry -> internet
    WireGuard {
        wg_entry: Option<Relay>,
        wg_exit: Relay,
        obfuscator: Option<Relay>,
    },
    /// Represents all relays generated for an OpenVPN tunnel.
    /// The traffic flows like this:
    ///     client -> bridge -> relay -> internet
    #[cfg(not(target_os = "android"))]
    OpenVpn { relay: Relay, bridge: Option<Relay> },
}
