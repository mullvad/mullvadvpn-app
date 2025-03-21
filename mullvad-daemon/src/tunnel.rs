use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    pin::Pin,
    str::FromStr,
    sync::Arc,
};

use tokio::sync::Mutex;

use mullvad_relay_selector::{GetRelay, RelaySelector, WireguardConfig};
use mullvad_types::{
    endpoint::MullvadWireguardEndpoint, location::GeoIpLocation, relay_list::Relay,
    settings::TunnelOptions,
};
use std::sync::LazyLock;
use talpid_core::tunnel_state_machine::TunnelParametersGenerator;
#[cfg(not(target_os = "android"))]
use talpid_types::net::{
    obfuscation::ObfuscatorConfig, openvpn, proxy::CustomProxy, wireguard, Endpoint,
    TunnelParameters,
};
#[cfg(target_os = "android")]
use talpid_types::net::{obfuscation::ObfuscatorConfig, wireguard, TunnelParameters};

use talpid_types::{net::IpAvailability, tunnel::ParameterGenerationError, ErrorExt};

use crate::device::{AccountManagerHandle, Error as DeviceError, PrivateAccountAndDevice};

/// The IP-addresses that the client uses when it connects to a server that supports the
/// "Same IP" functionality. This means all clients have the same in-tunnel IP on these
/// servers. This improves anonymity since the in-tunnel IP will not be unique to a specific
/// peer.
static SAME_IP_V4: LazyLock<IpAddr> =
    LazyLock::new(|| Ipv4Addr::from_str("10.127.255.254").unwrap().into());
static SAME_IP_V6: LazyLock<IpAddr> = LazyLock::new(|| {
    Ipv6Addr::from_str("fc00:bbbb:bbbb:bb01:ffff:ffff:ffff:ffff")
        .unwrap()
        .into()
});

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
        match relays {
            LastSelectedRelays::WireGuard {
                server_override, ..
            } => *server_override,
            #[cfg(not(target_os = "android"))]
            LastSelectedRelays::OpenVpn {
                server_override, ..
            } => *server_override,
        }
    }

    /// Gets the location associated with the last generated tunnel parameters.
    pub async fn get_last_location(&self) -> Option<GeoIpLocation> {
        let inner = self.0.lock().await;

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
                ..
            } => {
                entry_hostname = take_hostname(entry);
                hostname = exit.hostname.clone();
                obfuscator_hostname = take_hostname(obfuscator);
                bridge_hostname = None;
                location = exit.location.clone();
            }
            #[cfg(not(target_os = "android"))]
            LastSelectedRelays::OpenVpn { relay, bridge, .. } => {
                hostname = relay.hostname.clone();
                bridge_hostname = take_hostname(bridge);
                entry_hostname = None;
                obfuscator_hostname = None;
                location = relay.location.clone();
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
            #[cfg(not(target_os = "android"))]
            GetRelay::OpenVpn {
                endpoint,
                exit,
                bridge,
            } => {
                let bridge_relay = bridge.as_ref().and_then(|bridge| bridge.relay());
                let server_override = {
                    let first_relay = bridge_relay.unwrap_or(&exit);
                    (first_relay.overridden_ipv4 && endpoint.address.is_ipv4())
                        || (first_relay.overridden_ipv6 && endpoint.address.is_ipv6())
                };
                self.last_generated_relays = Some(LastSelectedRelays::OpenVpn {
                    relay: exit.clone(),
                    bridge: bridge_relay.cloned(),
                    server_override,
                });
                let bridge_settings = bridge.map(|bridge| bridge.to_proxy());
                Ok(self.create_openvpn_tunnel_parameters(endpoint, data, bridge_settings))
            }
            GetRelay::Wireguard {
                endpoint,
                obfuscator,
                inner,
            } => {
                let (obfuscator_relay, obfuscator_config) = match obfuscator {
                    Some(obfuscator) => (Some(obfuscator.relay), Some(obfuscator.config)),
                    None => (None, None),
                };

                let (wg_entry, wg_exit) = match inner {
                    WireguardConfig::Singlehop { exit } => (None, exit),
                    WireguardConfig::Multihop { exit, entry } => (Some(entry), exit),
                };
                let server_override = {
                    let first_relay = wg_entry.as_ref().unwrap_or(&wg_exit);
                    (first_relay.overridden_ipv4 && endpoint.peer.endpoint.is_ipv4())
                        || (first_relay.overridden_ipv6 && endpoint.peer.endpoint.is_ipv6())
                };

                self.last_generated_relays = Some(LastSelectedRelays::WireGuard {
                    wg_entry,
                    wg_exit,
                    obfuscator: obfuscator_relay,
                    server_override,
                });

                Ok(self.create_wireguard_tunnel_parameters(endpoint, data, obfuscator_config))
            }
            GetRelay::Custom(custom_relay) => {
                self.last_generated_relays = None;
                custom_relay
                     // TODO: generate proxy settings for custom tunnels
                     .to_tunnel_parameters(self.tunnel_options.clone(), None)
                     .map_err(|e| {
                         log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                         Error::ResolveCustomHostname
                     })
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    fn create_openvpn_tunnel_parameters(
        &self,
        endpoint: Endpoint,
        data: PrivateAccountAndDevice,
        bridge_settings: Option<CustomProxy>,
    ) -> TunnelParameters {
        openvpn::TunnelParameters {
            config: openvpn::ConnectionConfig::new(endpoint, data.account_number, "-".to_string()),
            options: self.tunnel_options.openvpn.clone(),
            generic_options: self.tunnel_options.generic.clone(),
            proxy: bridge_settings,
            #[cfg(target_os = "linux")]
            fwmark: mullvad_types::TUNNEL_FWMARK,
        }
        .into()
    }

    fn create_wireguard_tunnel_parameters(
        &self,
        endpoint: MullvadWireguardEndpoint,
        data: PrivateAccountAndDevice,
        obfuscator_config: Option<ObfuscatorConfig>,
    ) -> TunnelParameters {
        let tunnel_ipv4 = data.device.wg_data.addresses.ipv4_address.ip();
        let tunnel_ipv6 = data.device.wg_data.addresses.ipv6_address.ip();
        let tunnel = wireguard::TunnelConfig {
            private_key: data.device.wg_data.private_key,
            addresses: vec![IpAddr::from(tunnel_ipv4), IpAddr::from(tunnel_ipv6)],
        };
        // FIXME: Used for debugging purposes during the migration to same IP. Remove when
        // the migration is over.
        if tunnel_ipv4 == *SAME_IP_V4 || tunnel_ipv6 == *SAME_IP_V6 {
            log::debug!("Same IP is being used");
        } else {
            log::debug!("Same IP is NOT being used");
        }

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
        .into()
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
        ip_availbility: IpAvailability,
    ) -> Pin<Box<dyn Future<Output = Result<TunnelParameters, ParameterGenerationError>>>> {
        let generator = self.0.clone();
        Box::pin(async move {
            let mut inner = generator.lock().await;
            inner
                .generate(retry_attempt, ip_availbility)
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
                ParameterGenerationError::CustomTunnelHostResultionError
            }
            Error::NoAuthDetails | Error::SelectRelay(_) | Error::Device(_) => {
                ParameterGenerationError::NoMatchingRelay
            }
        }
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
        server_override: bool,
    },
    /// Represents all relays generated for an OpenVPN tunnel.
    /// The traffic flows like this:
    ///     client -> bridge -> relay -> internet
    #[cfg(not(target_os = "android"))]
    OpenVpn {
        relay: Relay,
        bridge: Option<Relay>,
        server_override: bool,
    },
}
