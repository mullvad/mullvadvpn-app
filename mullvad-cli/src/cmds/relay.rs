use clap::Subcommand;
use itertools::Itertools;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    location::Hostname,
    relay_constraints::{
        Constraint, LocationConstraint, Match, OpenVpnConstraints, Ownership, Provider, Providers,
        RelayConstraintsUpdate, RelaySettings, RelaySettingsUpdate, TransportPort,
        WireguardConstraints,
    },
    relay_list::{RelayEndpointData, RelayListCountry},
    ConnectionConfig, CustomTunnelEndpoint,
};
use std::{
    io::BufRead,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{
    all_of_the_internet, openvpn, wireguard, Endpoint, IpVersion, TransportProtocol, TunnelType,
};

use crate::{Error, Result};

use super::{on_off_parser, relay_constraints::LocationArgs};

#[derive(Subcommand, Debug)]
pub enum Relay {
    /// Display the current relay constraints
    Get,

    /// Set relay constraints, such as location and port
    #[clap(subcommand)]
    Set(SetCommands),

    /// List available relays
    List,

    /// Update the relay list
    Update,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Set country or city to select relays from. Use the 'list'
    /// command to show available alternatives.
    Location(LocationArgs),

    /// Set the location using only a hostname
    Hostname {
        /// A hostname, such as "se3-wireguard".
        hostname: Hostname,
    },

    /// Set hosting provider(s) to select relays from. The 'list'
    /// command shows the available relays and their providers.
    Provider {
        #[arg(required(true), num_args = 1..)]
        providers: Vec<Provider>,
    },

    /// Filter relays based on ownership. The 'list' command
    /// shows the available relays and whether they're rented.
    Ownership {
        /// Servers to select from: 'any', 'owned', or 'rented'.
        ownership: Constraint<Ownership>,
    },

    /// Set tunnel protocol specific constraints
    #[clap(subcommand)]
    Tunnel(SetTunnelCommands),

    /// Set tunnel protocol to use: 'any', 'wireguard', or 'openvpn'.
    TunnelProtocol { protocol: Constraint<TunnelType> },

    /// Set a custom VPN relay to use
    #[clap(subcommand)]
    Custom(SetCustomCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetTunnelCommands {
    /// Set OpenVPN-specific constraints
    #[clap(arg_required_else_help = true)]
    Openvpn {
        /// Port to use, or 'any'
        #[arg(long, short = 'p', requires = "transport_protocol")]
        port: Option<Constraint<u16>>,

        /// Transport protocol to use, or 'any'
        #[arg(long, short = 't')]
        transport_protocol: Option<Constraint<TransportProtocol>>,
    },

    /// Set WireGuard-specific constraints
    #[clap(arg_required_else_help = true)]
    Wireguard {
        /// Port to use, or 'any'
        #[arg(long, short = 'p')]
        port: Option<Constraint<u16>>,

        /// IP protocol to use, or 'any'
        #[arg(long, short = 'i')]
        ip_version: Option<Constraint<IpVersion>>,

        /// Whether to enable multihop. The location constraints are specified with
        /// 'entry-location'.
        #[arg(long, short = 'm', value_parser = on_off_parser())]
        use_multihop: Option<bool>,

        #[clap(subcommand)]
        entry_location: Option<EntryLocation>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum EntryLocation {
    /// Entry endpoint to use. This can be 'any' or any location that is valid with 'set location',
    /// such as 'se got'.
    EntryLocation(LocationArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCustomCommands {
    /// Use a custom OpenVPN relay
    #[clap(arg_required_else_help = true)]
    Openvpn {
        /// Hostname or IP
        host: String,
        /// Remote port
        port: u16,
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
        /// Transport protocol to use
        #[arg(default_value_t = TransportProtocol::Udp)]
        transport_protocol: TransportProtocol,
    },

    /// Use a custom WireGuard relay
    #[clap(arg_required_else_help = true)]
    Wireguard {
        /// Hostname or IP
        host: String,
        /// Remote port
        port: u16,
        /// Base64 encoded public key of remote peer
        #[arg(value_parser = wireguard::PublicKey::from_base64)]
        peer_pubkey: wireguard::PublicKey,
        /// IP addresses of local tunnel interface
        #[arg(required = true, num_args = 1..)]
        tunnel_ip: Vec<IpAddr>,
        /// IPv4 gateway address
        #[arg(long)]
        v4_gateway: Ipv4Addr,
        /// IPv6 gateway address
        #[arg(long)]
        v6_gateway: Option<Ipv6Addr>,
    },
}

impl Relay {
    pub async fn handle(self) -> Result<()> {
        match self {
            Relay::Get => Self::get().await,
            Relay::List => Self::list().await,
            Relay::Update => Self::update().await,
            Relay::Set(subcmd) => Self::set(subcmd).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_settings = rpc.get_settings().await?.relay_settings;
        println!("Current constraints: {relay_settings}");
        Ok(())
    }

    async fn list() -> Result<()> {
        let mut countries = Self::get_filtered_relays().await?;
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
                    let support_msg = match relay.endpoint_data {
                        RelayEndpointData::Openvpn => "OpenVPN",
                        RelayEndpointData::Wireguard(_) => "WireGuard",
                        _ => unreachable!("Bug in relay filtering earlier on"),
                    };
                    let ownership = if relay.owned {
                        "Mullvad-owned"
                    } else {
                        "rented"
                    };
                    let mut addresses: Vec<IpAddr> = vec![relay.ipv4_addr_in.into()];
                    if let Some(ipv6_addr) = relay.ipv6_addr_in {
                        addresses.push(ipv6_addr.into());
                    }
                    println!(
                        "\t\t{} ({}) - {}, hosted by {} ({ownership})",
                        relay.hostname,
                        addresses.iter().join(", "),
                        support_msg,
                        relay.provider
                    );
                }
            }
            println!();
        }
        Ok(())
    }

    async fn update() -> Result<()> {
        MullvadProxyClient::new()
            .await?
            .update_relay_locations()
            .await?;
        println!("Updating relay list in the background...");
        Ok(())
    }

    async fn get_filtered_relays() -> Result<Vec<RelayListCountry>> {
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_list = rpc.get_relay_locations().await?;

        let mut countries = vec![];

        for mut country in relay_list.countries {
            country.cities = country
                .cities
                .into_iter()
                .filter_map(|mut city| {
                    city.relays.retain(|relay| {
                        relay.active && relay.endpoint_data != RelayEndpointData::Bridge
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

        Ok(countries)
    }

    async fn update_constraints(update: RelaySettingsUpdate) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.update_relay_settings(update).await?;
        println!("Relay constraints updated");
        Ok(())
    }

    async fn set(subcmd: SetCommands) -> Result<()> {
        match subcmd {
            SetCommands::Custom(subcmd) => Self::set_custom(subcmd).await,
            SetCommands::Location(location) => Self::set_location(location).await,
            SetCommands::Hostname { hostname } => Self::set_hostname(hostname).await,
            SetCommands::Provider { providers } => Self::set_providers(providers).await,
            SetCommands::Ownership { ownership } => Self::set_ownership(ownership).await,
            SetCommands::Tunnel(subcmd) => Self::set_tunnel(subcmd).await,
            SetCommands::TunnelProtocol { protocol } => Self::set_tunnel_protocol(protocol).await,
        }
    }

    async fn set_tunnel(subcmd: SetTunnelCommands) -> Result<()> {
        match subcmd {
            SetTunnelCommands::Openvpn {
                port,
                transport_protocol,
            } => Self::set_openvpn_constraints(port, transport_protocol).await,
            SetTunnelCommands::Wireguard {
                port,
                ip_version,
                use_multihop,
                entry_location,
            } => {
                Self::set_wireguard_constraints(port, ip_version, use_multihop, entry_location)
                    .await
            }
        }
    }

    async fn set_custom(subcmd: SetCustomCommands) -> Result<()> {
        let custom_endpoint = match subcmd {
            SetCustomCommands::Openvpn {
                host,
                port,
                username,
                password,
                transport_protocol,
            } => {
                Self::read_custom_openvpn_relay(host, port, username, password, transport_protocol)
            }
            SetCustomCommands::Wireguard {
                host,
                port,
                peer_pubkey,
                tunnel_ip,
                v4_gateway,
                v6_gateway,
            } => {
                Self::read_custom_wireguard_relay(
                    host,
                    port,
                    peer_pubkey,
                    tunnel_ip,
                    v4_gateway,
                    v6_gateway,
                )
                .await?
            }
        };
        Self::update_constraints(RelaySettingsUpdate::CustomTunnelEndpoint(custom_endpoint)).await
    }

    fn read_custom_openvpn_relay(
        host: String,
        port: u16,
        username: String,
        password: String,
        protocol: TransportProtocol,
    ) -> CustomTunnelEndpoint {
        CustomTunnelEndpoint {
            host,
            config: ConnectionConfig::OpenVpn(openvpn::ConnectionConfig {
                endpoint: Endpoint::from_socket_address(
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
                    protocol,
                ),
                username,
                password,
            }),
        }
    }

    async fn read_custom_wireguard_relay(
        host: String,
        port: u16,
        peer_pubkey: wireguard::PublicKey,
        tunnel_ip: Vec<IpAddr>,
        ipv4_gateway: Ipv4Addr,
        ipv6_gateway: Option<Ipv6Addr>,
    ) -> Result<CustomTunnelEndpoint> {
        println!("Reading private key from standard input");

        let private_key_str = tokio::task::spawn_blocking(|| {
            let mut private_key_str = String::new();
            let _ = std::io::stdin().lock().read_line(&mut private_key_str);
            if private_key_str.trim().is_empty() {
                eprintln!("Expected to read private key from standard input");
            }
            private_key_str
        })
        .await
        .unwrap();

        let private_key = wireguard::PrivateKey::from_base64(&private_key_str)
            .map_err(|_| Error::InvalidCommand("invalid private key"))?;

        Ok(CustomTunnelEndpoint {
            host,
            config: ConnectionConfig::Wireguard(wireguard::ConnectionConfig {
                tunnel: wireguard::TunnelConfig {
                    private_key,
                    addresses: tunnel_ip,
                },
                peer: wireguard::PeerConfig {
                    public_key: peer_pubkey,
                    allowed_ips: all_of_the_internet(),
                    endpoint: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
                    psk: None,
                },
                exit_peer: None,
                ipv4_gateway,
                ipv6_gateway,
                // NOTE: Ignored in gRPC
                #[cfg(target_os = "linux")]
                fwmark: None,
            }),
        })
    }

    async fn set_hostname(hostname: String) -> Result<()> {
        let countries = Self::get_filtered_relays().await?;

        let find_relay = || {
            for country in countries {
                for city in country.cities {
                    for relay in city.relays {
                        if relay.hostname.to_lowercase() == hostname.to_lowercase() {
                            return Some(LocationConstraint::Hostname(
                                country.code,
                                city.code,
                                relay.hostname,
                            ));
                        }
                    }
                }
            }
            None
        };

        let location = find_relay().ok_or(Error::InvalidCommand("hostname not found"))?;

        println!("Setting location constraint to {location}");
        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(Constraint::Only(location)),
            ..Default::default()
        }))
        .await
    }

    async fn set_location(location_constraint: LocationArgs) -> Result<()> {
        let location_constraint = Constraint::from(location_constraint);
        match &location_constraint {
            Constraint::Any => (),
            Constraint::Only(constraint) => {
                let countries = Self::get_filtered_relays().await?;

                let found = countries
                    .into_iter()
                    .flat_map(|country| country.cities)
                    .flat_map(|city| city.relays)
                    .any(|relay| constraint.matches(&relay));

                if !found {
                    eprintln!("Warning: No matching relay was found.");
                }
            }
        }
        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(location_constraint),
            ..Default::default()
        }))
        .await
    }

    async fn set_providers(providers: Vec<String>) -> Result<()> {
        let providers = if providers[0].eq_ignore_ascii_case("any") {
            Constraint::Any
        } else {
            Constraint::Only(Providers::new(providers.into_iter()).unwrap())
        };
        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            providers: Some(providers),
            ..Default::default()
        }))
        .await
    }

    async fn set_ownership(ownership: Constraint<Ownership>) -> Result<()> {
        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            ownership: Some(ownership),
            ..Default::default()
        }))
        .await
    }

    async fn set_openvpn_constraints(
        port: Option<Constraint<u16>>,
        protocol: Option<Constraint<TransportProtocol>>,
    ) -> Result<()> {
        let mut openvpn_constraints = {
            let mut rpc = MullvadProxyClient::new().await?;
            Self::get_openvpn_constraints(&mut rpc).await?
        };
        openvpn_constraints.port =
            parse_transport_port(port, protocol, &mut openvpn_constraints.port);

        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            openvpn_constraints: Some(openvpn_constraints),
            ..Default::default()
        }))
        .await
    }

    async fn get_openvpn_constraints(rpc: &mut MullvadProxyClient) -> Result<OpenVpnConstraints> {
        match rpc.get_settings().await?.relay_settings {
            RelaySettings::Normal(settings) => Ok(settings.openvpn_constraints),
            RelaySettings::CustomTunnelEndpoint(_settings) => {
                println!("Clearing custom tunnel constraints");
                Ok(OpenVpnConstraints::default())
            }
        }
    }

    async fn set_wireguard_constraints(
        port: Option<Constraint<u16>>,
        ip_version: Option<Constraint<IpVersion>>,
        use_multihop: Option<bool>,
        entry_location: Option<EntryLocation>,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let wireguard = rpc.get_relay_locations().await?.wireguard;
        let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

        if let Some(port) = port {
            wireguard_constraints.port = match port {
                Constraint::Any => Constraint::Any,
                Constraint::Only(specific_port) => {
                    let is_valid_port = wireguard
                        .port_ranges
                        .into_iter()
                        .any(|(first, last)| first <= specific_port && specific_port <= last);
                    if !is_valid_port {
                        return Err(Error::CommandFailed("The specified port is invalid"));
                    }
                    Constraint::Only(specific_port)
                }
            }
        }

        if let Some(ipv) = ip_version {
            wireguard_constraints.ip_version = ipv;
        }
        if let Some(use_multihop) = use_multihop {
            wireguard_constraints.use_multihop = use_multihop;
        }
        if let Some(EntryLocation::EntryLocation(entry)) = entry_location {
            wireguard_constraints.entry_location = Constraint::from(entry);
        }

        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            wireguard_constraints: Some(wireguard_constraints),
            ..Default::default()
        }))
        .await
    }

    async fn get_wireguard_constraints(
        rpc: &mut MullvadProxyClient,
    ) -> Result<WireguardConstraints> {
        match rpc.get_settings().await?.relay_settings {
            RelaySettings::Normal(settings) => Ok(settings.wireguard_constraints),
            RelaySettings::CustomTunnelEndpoint(_settings) => {
                println!("Clearing custom tunnel constraints");
                Ok(WireguardConstraints::default())
            }
        }
    }

    async fn set_tunnel_protocol(protocol: Constraint<TunnelType>) -> Result<()> {
        Self::update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            tunnel_protocol: Some(protocol),
            ..Default::default()
        }))
        .await
    }
}

fn parse_transport_port(
    port: Option<Constraint<u16>>,
    protocol: Option<Constraint<TransportProtocol>>,
    current_constraint: &mut Constraint<TransportPort>,
) -> Constraint<TransportPort> {
    let port = match port {
        Some(port) => port,
        None => current_constraint
            .map(|p| p.port)
            .unwrap_or(Constraint::Any),
    };
    let protocol = match protocol {
        Some(protocol) => protocol,
        None => current_constraint.map(|p| p.protocol),
    };
    match (port, protocol) {
        (port, Constraint::Any) => {
            if port.is_only() {
                println!("The port constraint was set to 'any'");
            }
            Constraint::Any
        }
        (port, Constraint::Only(protocol)) => Constraint::Only(TransportPort { protocol, port }),
    }
}
