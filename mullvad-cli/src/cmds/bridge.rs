use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint, Ownership,
        Provider, Providers,
    },
    relay_list::RelayEndpointData,
};
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::openvpn::{self, SHADOWSOCKS_CIPHERS};

use crate::{Error, Result};

use super::relay_constraints::LocationArgs;

#[derive(Subcommand, Debug)]
pub enum Bridge {
    /// Get current bridge settings
    Get,

    /// Set bridge state and settings, such as provider
    #[clap(subcommand)]
    Set(SetCommands),

    /// List available bridge relays
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Specify whether to use a bridge
    State { policy: BridgeState },

    /// Set country or city to select relays from. Use the 'list'
    /// command to show available alternatives.
    Location(LocationArgs),

    /// Set hosting provider(s) to select relays from. The 'list'
    /// command shows the available relays and their providers.
    Providers {
        #[arg(required(true), num_args = 1..)]
        providers: Vec<Provider>,
    },

    /// Filter relays based on ownership. The 'list' command
    /// shows the available relays and whether they're rented.
    Ownership {
        /// Servers to select from: 'any', 'owned', or 'rented'.
        ownership: Constraint<Ownership>,
    },

    /// Configure a SOCKS5 proxy
    #[clap(subcommand)]
    Custom(SetCustomCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCustomCommands {
    /// Configure a local SOCKS5 proxy
    #[cfg_attr(
        target_os = "linux",
        clap(
            about = "Registers a local SOCKS5 proxy. The server must be excluded using \
        'mullvad-exclude', or `SO_MARK` must be set to '0x6d6f6c65', in order \
        to bypass firewall restrictions"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        clap(
            about = "Registers a local SOCKS5 proxy. The server must run as root to bypass \
        firewall restrictions"
        )
    )]
    Local {
        /// The port that the server on localhost is listening on
        local_port: u16,

        /// The IP of the remote peer
        remote_ip: IpAddr,

        /// The port of the remote peer
        remote_port: u16,
    },

    /// Configure a remote SOCKS5 proxy
    Remote {
        /// The IP of the remote proxy server
        remote_ip: IpAddr,

        /// The port of the remote proxy server
        remote_port: u16,

        /// Username for authentication
        username: Option<String>,

        /// Password for authentication
        password: Option<String>,
    },

    /// Configure bundled Shadowsocks proxy
    Shadowsocks {
        /// The IP of the remote Shadowsocks server
        remote_ip: IpAddr,

        /// The port of the remote Shadowsocks server
        #[arg(default_value = "443")]
        remote_port: u16,

        /// Password for authentication
        #[arg(default_value = "mullvad")]
        password: String,

        /// Password for authentication
        #[arg(value_parser = check_cipher, default_value = "aes-256-gcm")]
        cipher: String,
    },
}

pub fn check_cipher(cipher: &str) -> Result<String> {
    if SHADOWSOCKS_CIPHERS.contains(&cipher) {
        return Ok(cipher.to_string());
    }
    eprintln!(
        "The following ciphers are valid: {}",
        SHADOWSOCKS_CIPHERS.join(" ")
    );
    Err(Error::InvalidCommand("invalid cipher"))
}

impl Bridge {
    pub async fn handle(self) -> Result<()> {
        match self {
            Bridge::Get => Self::get().await,
            Bridge::List => Self::list().await,
            Bridge::Set(subcmd) => Self::set(subcmd).await,
        }
    }

    async fn set(subcmd: SetCommands) -> Result<()> {
        match subcmd {
            SetCommands::State { policy } => {
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_bridge_state(policy).await?;
                println!("Updated bridge state");
                Ok(())
            }
            SetCommands::Location(location) => {
                Self::update_bridge_settings(Some(Constraint::from(location)), None, None).await
            }
            SetCommands::Ownership { ownership } => {
                Self::update_bridge_settings(None, None, Some(ownership)).await
            }
            SetCommands::Providers { providers } => {
                let providers = if providers[0].eq_ignore_ascii_case("any") {
                    Constraint::Any
                } else {
                    Constraint::Only(Providers::new(providers.into_iter()).unwrap())
                };
                Self::update_bridge_settings(None, Some(providers), None).await
            }
            SetCommands::Custom(subcmd) => Self::set_custom(subcmd).await,
        }
    }

    async fn set_custom(subcmd: SetCustomCommands) -> Result<()> {
        match subcmd {
            SetCustomCommands::Local {
                local_port,
                remote_ip,
                remote_port,
            } => {
                let local_proxy = openvpn::LocalProxySettings {
                    port: local_port,
                    peer: SocketAddr::new(remote_ip, remote_port),
                };
                let packed_proxy = openvpn::ProxySettings::Local(local_proxy);
                if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                    panic!("{}", error);
                }

                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
                    .await?;
            }
            SetCustomCommands::Remote {
                remote_ip,
                remote_port,
                username,
                password,
            } => {
                let auth = match (username, password) {
                    (Some(username), Some(password)) => {
                        Some(openvpn::ProxyAuth { username, password })
                    }
                    _ => None,
                };
                let proxy = openvpn::RemoteProxySettings {
                    address: SocketAddr::new(remote_ip, remote_port),
                    auth,
                };
                let packed_proxy = openvpn::ProxySettings::Remote(proxy);
                if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                    panic!("{}", error);
                }

                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
                    .await?;
            }
            SetCustomCommands::Shadowsocks {
                remote_ip,
                remote_port,
                password,
                cipher,
            } => {
                let proxy = openvpn::ShadowsocksProxySettings {
                    peer: SocketAddr::new(remote_ip, remote_port),
                    password,
                    cipher,
                    #[cfg(target_os = "linux")]
                    fwmark: None,
                };
                let packed_proxy = openvpn::ProxySettings::Shadowsocks(proxy);
                if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                    panic!("{}", error);
                }

                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))
                    .await?;
            }
        }

        println!("Updated bridge settings");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        println!("Bridge state: {}", settings.bridge_state);
        match settings.bridge_settings {
            BridgeSettings::Custom(proxy) => match proxy {
                openvpn::ProxySettings::Local(local_proxy) => Self::print_local_proxy(&local_proxy),
                openvpn::ProxySettings::Remote(remote_proxy) => {
                    Self::print_remote_proxy(&remote_proxy)
                }
                openvpn::ProxySettings::Shadowsocks(shadowsocks_proxy) => {
                    Self::print_shadowsocks_proxy(&shadowsocks_proxy)
                }
            },
            BridgeSettings::Normal(constraints) => {
                println!("Bridge constraints: {constraints}")
            }
        };
        Ok(())
    }

    fn print_local_proxy(proxy: &openvpn::LocalProxySettings) {
        println!("proxy: local");
        println!("  local port: {}", proxy.port);
        println!("  peer address: {}", proxy.peer);
    }

    fn print_remote_proxy(proxy: &openvpn::RemoteProxySettings) {
        println!("proxy: remote");
        println!("  server address: {}", proxy.address);

        if let Some(ref auth) = proxy.auth {
            println!("  auth username: {}", auth.username);
            println!("  auth password: {}", auth.password);
        } else {
            println!("  auth: none");
        }
    }

    fn print_shadowsocks_proxy(proxy: &openvpn::ShadowsocksProxySettings) {
        println!("proxy: Shadowsocks");
        println!("  peer address: {}", proxy.peer);
        println!("  password: {}", proxy.password);
        println!("  cipher: {}", proxy.cipher);
    }

    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_list = rpc.get_relay_locations().await?;

        let mut countries = Vec::new();

        for mut country in relay_list.countries {
            country.cities = country
                .cities
                .into_iter()
                .filter_map(|mut city| {
                    city.relays.retain(|relay| {
                        relay.active && matches!(relay.endpoint_data, RelayEndpointData::Bridge)
                    });
                    if !city.relays.is_empty() {
                        Some(city)
                    } else {
                        None
                    }
                })
                .collect();
            if !country.cities.is_empty() {
                countries.push(country);
            }
        }

        countries.sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
        for mut country in countries {
            country
                .cities
                .sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
            println!("{} ({})", country.name, country.code);
            for mut city in country.cities {
                city.relays
                    .sort_by(|r1, r2| natord::compare_ignore_case(&r1.hostname, &r2.hostname));
                println!(
                    "\t{} ({}) @ {:.5}°N, {:.5}°W",
                    city.name, city.code, city.latitude, city.longitude
                );
                for relay in &city.relays {
                    let ownership = if relay.owned {
                        "Mullvad-owned"
                    } else {
                        "rented"
                    };
                    println!(
                        "\t\t{} ({}) - hosted by {} ({ownership})",
                        relay.hostname, relay.ipv4_addr_in, relay.provider
                    );
                }
            }
            println!();
        }
        Ok(())
    }

    async fn update_bridge_settings(
        location: Option<Constraint<LocationConstraint>>,
        providers: Option<Constraint<Providers>>,
        ownership: Option<Constraint<Ownership>>,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;

        let constraints = match rpc.get_settings().await?.bridge_settings {
            BridgeSettings::Normal(mut constraints) => {
                if let Some(new_location) = location {
                    constraints.location = new_location;
                }
                if let Some(new_providers) = providers {
                    constraints.providers = new_providers;
                }
                if let Some(new_ownership) = ownership {
                    constraints.ownership = new_ownership;
                }
                constraints
            }
            _ => BridgeConstraints {
                location: location.unwrap_or(Constraint::Any),
                providers: providers.unwrap_or(Constraint::Any),
                ownership: ownership.unwrap_or(Constraint::Any),
            },
        };

        rpc.set_bridge_settings(BridgeSettings::Normal(constraints))
            .await?;

        println!("Updated bridge settings");

        Ok(())
    }
}
