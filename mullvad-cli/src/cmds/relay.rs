use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use itertools::Itertools;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    location::{CountryCode, Location},
    relay_constraints::{
        Constraint, GeographicLocationConstraint, LocationConstraint, LocationConstraintFormatter,
        Match, OpenVpnConstraints, Ownership, Provider, Providers, RelayConstraints, RelayOverride,
        RelaySettings, TransportPort, WireguardConstraints,
    },
    relay_list::{RelayEndpointData, RelayList, RelayListCountry},
    ConnectionConfig, CustomTunnelEndpoint,
};
use std::{
    collections::HashMap,
    io::BufRead,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{
    all_of_the_internet, openvpn, wireguard, Endpoint, IpVersion, TransportProtocol, TunnelType,
};

use super::{relay_constraints::LocationArgs, BooleanOption};
use crate::{cmds::receive_confirmation, print_option};

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

    /// Override options for individual relays/servers
    #[clap(subcommand)]
    Override(OverrideCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Select a relay using country, city or hostname.
    /// The 'mullvad relay list' command shows the available relays and their
    /// geographical location.
    #[command(
        override_usage = "mullvad relay set location <COUNTRY> [CITY] [HOSTNAME] | <HOSTNAME>

  Select relay using a country:

\tmullvad relay set location se

  Select relay using a country and city:

\tmullvad relay set location se got

  Select relay using a country, city and hostname:

\tmullvad relay set location se got se-got-wg-004

  Select relay using only its hostname:

\tmullvad relay set location se-got-wg-004"
    )]
    Location(LocationArgs),

    /// Set custom list to select relays from. Use the 'custom-lists list'
    /// command to show available alternatives.
    CustomList {
        /// Name of the custom list to use
        custom_list_name: String,
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
        #[arg(long, short = 'm')]
        use_multihop: Option<BooleanOption>,

        #[clap(subcommand)]
        entry_location: Option<EntryLocation>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum EntryLocation {
    /// Entry endpoint to use. This can be 'any' or any location that is valid with 'set location',
    /// such as 'se got'.
    #[command(
        override_usage = "mullvad relay set tunnel wireguard entry-location <COUNTRY> [CITY] [HOSTNAME] | <HOSTNAME>

  Select entry location using a country:

\tmullvad relay set tunnel wireguard entry-location se

  Select entry location using a country and city:

\tmullvad relay set tunnel wireguard entry-location se got

  Select entry location using a country, city and hostname:

\tmullvad relay set tunnel wireguard entry-location se got se-got-wg-004

  Select entry location using only its hostname:

\tmullvad relay set tunnel wireguard entry-location se-got-wg-004"
    )]
    EntryLocation(LocationArgs),
    /// Name of custom list to use to pick entry endpoint.
    CustomList { custom_list_name: String },
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

#[derive(Subcommand, Debug, Clone)]
pub enum OverrideCommands {
    /// Show current custom fields for servers
    Get,
    /// Set a custom field for a server
    #[clap(subcommand)]
    Set(OverrideSetCommands),
    /// Unset a custom field for a server
    #[clap(subcommand)]
    Unset(OverrideUnsetCommands),
    /// Unset custom IPs for all servers
    ClearAll {
        /// Clear overrides without asking for confirmation
        #[arg(long, short = 'y', default_value_t = false)]
        confirm: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum OverrideSetCommands {
    /// Override entry IPv4 address for a given relay
    Ipv4 {
        /// The unique hostname for the server to set the override on
        hostname: String,
        /// The IPv4 address to use to connect to this server
        address: Ipv4Addr,
    },
    /// Override entry IPv6 address for a given relay
    Ipv6 {
        /// The unique hostname for the server to set the override on
        hostname: String,
        /// The IPv6 address to use to connect to this server
        address: Ipv6Addr,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum OverrideUnsetCommands {
    /// Remove overridden entry IPv4 address for the given server
    Ipv4 { hostname: String },
    /// Remove overridden entry IPv6 address for the given server
    Ipv6 { hostname: String },
}

impl Relay {
    pub async fn handle(self) -> Result<()> {
        match self {
            Relay::Get => Self::get().await,
            Relay::List => Self::list().await,
            Relay::Update => Self::update().await,
            Relay::Set(subcmd) => Self::set(subcmd).await,
            Relay::Override(subcmd) => Self::r#override(subcmd).await,
        }
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;
        let relay_settings = settings.relay_settings;

        match relay_settings {
            RelaySettings::CustomTunnelEndpoint(endpoint) => {
                println!("Custom endpoint: {endpoint}")
            }

            RelaySettings::Normal(constraints) => {
                println!("Generic constraints");

                print_option!(
                    "Location",
                    constraints
                        .location
                        .as_ref()
                        .map(|location| LocationConstraintFormatter {
                            constraint: location,
                            custom_lists: &settings.custom_lists
                        }),
                );

                print_option!("Tunnel protocol", constraints.tunnel_protocol,);

                print_option!("Provider(s)", constraints.providers,);
                print_option!("Ownership", constraints.ownership,);

                println!("OpenVPN constraints");

                match constraints.openvpn_constraints.port {
                    Constraint::Any => {
                        print_option!("Port", "any",);
                        print_option!("Transport", "any",);
                    }
                    Constraint::Only(transport_port) => {
                        print_option!("Port", transport_port.port,);
                        print_option!("Transport", transport_port.protocol,);
                    }
                }

                println!("WireGuard constraints");

                print_option!("Port", constraints.wireguard_constraints.port,);

                print_option!("IP protocol", constraints.wireguard_constraints.ip_version,);

                print_option!(
                    "Multihop state",
                    if constraints.wireguard_constraints.use_multihop {
                        "enabled"
                    } else {
                        "disabled"
                    },
                );
                print_option!(
                    "Multihop entry",
                    constraints
                        .wireguard_constraints
                        .entry_location
                        .as_ref()
                        .map(|location| LocationConstraintFormatter {
                            constraint: location,
                            custom_lists: &settings.custom_lists
                        }),
                );
            }
        }

        Ok(())
    }

    async fn list() -> Result<()> {
        let mut countries = get_filtered_relays().await?;
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

    /// Get active relays which are not bridges.

    async fn update_constraints(update_fn: impl FnOnce(&mut RelayConstraints)) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;

        let relay_settings = settings.get_relay_settings();
        let mut constraints = match relay_settings {
            RelaySettings::Normal(normal) => normal,
            RelaySettings::CustomTunnelEndpoint(_custom) => {
                println!("Removing custom relay settings");
                RelayConstraints::default()
            }
        };
        update_fn(&mut constraints);
        rpc.set_relay_settings(RelaySettings::Normal(constraints))
            .await?;
        println!("Relay constraints updated");
        Ok(())
    }

    async fn set(subcmd: SetCommands) -> Result<()> {
        match subcmd {
            SetCommands::Custom(subcmd) => Self::set_custom(subcmd).await,
            SetCommands::Location(location) => Self::set_location(location).await,
            SetCommands::CustomList { custom_list_name } => {
                Self::set_custom_list(custom_list_name).await
            }
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
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_relay_settings(RelaySettings::CustomTunnelEndpoint(custom_endpoint))
            .await?;
        println!("Relay constraints updated");
        Ok(())
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
            let private_key_str = private_key_str.trim().to_owned();
            if private_key_str.is_empty() {
                eprintln!("Expected to read private key from standard input");
            }
            private_key_str
        })
        .await
        .unwrap();

        let private_key =
            wireguard::PrivateKey::from_base64(&private_key_str).context("Invalid private key")?;

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

    async fn set_location(location_constraint_args: LocationArgs) -> Result<()> {
        let countries = get_filtered_relays().await?;
        let constraint =
            if let Some(relay) =
                // The country field is assumed to be hostname due to CLI argument parsing
                find_relay_by_hostname(&countries, &location_constraint_args.country)
            {
                Constraint::Only(LocationConstraint::Location(relay))
            } else {
                let location_constraint: Constraint<GeographicLocationConstraint> =
                    Constraint::from(location_constraint_args);
                match &location_constraint {
                    Constraint::Any => (),
                    Constraint::Only(constraint) => {
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
                location_constraint.map(LocationConstraint::Location)
            };

        Self::update_constraints(|constraints| {
            constraints.location = constraint;
        })
        .await
    }

    async fn set_custom_list(custom_list_name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let list_id = super::custom_list::find_list_by_name(&mut rpc, &custom_list_name)
            .await?
            .id;
        Self::update_constraints(|constraints| {
            constraints.location = Constraint::Only(LocationConstraint::CustomList { list_id });
        })
        .await
    }

    async fn set_providers(providers: Vec<String>) -> Result<()> {
        let providers = if providers[0].eq_ignore_ascii_case("any") {
            Constraint::Any
        } else {
            Constraint::Only(Providers::new(providers.into_iter()).unwrap())
        };
        Self::update_constraints(|constraints| {
            constraints.providers = providers;
        })
        .await
    }

    async fn set_ownership(ownership: Constraint<Ownership>) -> Result<()> {
        Self::update_constraints(|constraints| {
            constraints.ownership = ownership;
        })
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
        openvpn_constraints.port = parse_transport_port(port, protocol, &openvpn_constraints.port);

        Self::update_constraints(|constraints| {
            constraints.openvpn_constraints = openvpn_constraints;
        })
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
        use_multihop: Option<BooleanOption>,
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
                        return Err(anyhow!("The specified port is invalid"));
                    }
                    Constraint::Only(specific_port)
                }
            }
        }

        if let Some(ipv) = ip_version {
            wireguard_constraints.ip_version = ipv;
        }
        if let Some(use_multihop) = use_multihop {
            wireguard_constraints.use_multihop = *use_multihop;
        }
        match entry_location {
            Some(EntryLocation::EntryLocation(entry)) => {
                let countries = get_filtered_relays().await?;
                // The country field is assumed to be hostname due to CLI argument parsing
                wireguard_constraints.entry_location =
                    if let Some(relay) = find_relay_by_hostname(&countries, &entry.country) {
                        Constraint::Only(LocationConstraint::Location(relay))
                    } else {
                        Constraint::from(entry)
                    };
            }
            Some(EntryLocation::CustomList { custom_list_name }) => {
                let list_id = super::custom_list::find_list_by_name(&mut rpc, &custom_list_name)
                    .await?
                    .id;
                wireguard_constraints.entry_location =
                    Constraint::Only(LocationConstraint::CustomList { list_id });
            }
            None => (),
        }

        Self::update_constraints(|constraints| {
            constraints.wireguard_constraints = wireguard_constraints;
        })
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
        Self::update_constraints(|constraints| {
            constraints.tunnel_protocol = protocol;
        })
        .await
    }

    async fn update_override(
        hostname: &str,
        update_fn: impl FnOnce(&mut RelayOverride),
        warn_non_existent_hostname: bool,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;

        if warn_non_existent_hostname {
            let relay_list = rpc.get_relay_locations().await?;
            let countries = filter_relays(relay_list);
            if find_relay_by_hostname(&countries, hostname).is_none() {
                eprintln!("Warning: Setting overrides for an unrecognized server");
            }
        }

        let mut relay_overrides = settings.relay_overrides;
        let mut element = relay_overrides
            .iter()
            .position(|elem| elem.hostname == hostname)
            .map(|index| relay_overrides.swap_remove(index))
            .unwrap_or_else(|| RelayOverride::empty(hostname.to_owned()));
        update_fn(&mut element);

        rpc.set_relay_override(element).await?;
        println!("Updated override options for {hostname}");
        Ok(())
    }

    async fn r#override(subcmd: OverrideCommands) -> Result<()> {
        match subcmd {
            OverrideCommands::Get => {
                let mut rpc = MullvadProxyClient::new().await?;
                let settings = rpc.get_settings().await?;

                let mut overrides = HashMap::new();
                for relay_override in settings.relay_overrides {
                    overrides.insert(relay_override.hostname.clone(), relay_override);
                }

                struct Country {
                    name: String,
                    code: CountryCode,
                    cities: Vec<City>,
                }
                struct City {
                    name: String,
                    code: CountryCode,
                    overrides: Vec<RelayOverride>,
                }

                let mut countries_with_overrides = vec![];
                for country in get_filtered_relays().await? {
                    let mut country_with_overrides = Country {
                        name: country.name,
                        code: country.code,
                        cities: vec![],
                    };

                    for city in country.cities {
                        let mut city_with_overrides = City {
                            name: city.name,
                            code: city.code,
                            overrides: vec![],
                        };

                        for relay in city.relays {
                            if let Some(relay_override) = overrides.remove(&relay.hostname) {
                                city_with_overrides.overrides.push(relay_override);
                            }
                        }

                        if !city_with_overrides.overrides.is_empty() {
                            country_with_overrides.cities.push(city_with_overrides);
                        }
                    }

                    if !country_with_overrides.cities.is_empty() {
                        countries_with_overrides.push(country_with_overrides);
                    }
                }

                let print_relay_override = |relay_override: RelayOverride| {
                    println!("{:<8}{}:", " ", relay_override.hostname);
                    if let Some(ipv4) = relay_override.ipv4_addr_in {
                        println!("{:<12}ipv4: {ipv4}", " ");
                    }
                    if let Some(ipv6) = relay_override.ipv6_addr_in {
                        println!("{:<12}ipv6: {ipv6}", " ");
                    }
                };

                for country in countries_with_overrides {
                    println!("{} ({})", country.name, country.code);
                    for city in country.cities {
                        println!("{:<4}{} ({})", " ", city.name, city.code);
                        for relay_override in city.overrides {
                            print_relay_override(relay_override);
                        }
                    }
                }

                if !overrides.is_empty() {
                    println!("Overrides for unrecognized servers. Consider removing these!");
                    for relay_override in overrides.into_values() {
                        print_relay_override(relay_override);
                    }
                }
            }
            OverrideCommands::Set(set_cmds) => match set_cmds {
                OverrideSetCommands::Ipv4 { hostname, address } => {
                    Self::update_override(
                        &hostname,
                        |relay_override| relay_override.ipv4_addr_in = Some(address),
                        true,
                    )
                    .await?;
                }
                OverrideSetCommands::Ipv6 { hostname, address } => {
                    Self::update_override(
                        &hostname,
                        |relay_override| relay_override.ipv6_addr_in = Some(address),
                        true,
                    )
                    .await?;
                }
            },
            OverrideCommands::Unset(cmds) => match cmds {
                OverrideUnsetCommands::Ipv4 { hostname } => {
                    Self::update_override(
                        &hostname,
                        |relay_override| relay_override.ipv4_addr_in = None,
                        false,
                    )
                    .await?;
                }
                OverrideUnsetCommands::Ipv6 { hostname } => {
                    Self::update_override(
                        &hostname,
                        |relay_override| relay_override.ipv6_addr_in = None,
                        false,
                    )
                    .await?;
                }
            },
            OverrideCommands::ClearAll { confirm } => {
                if confirm
                    || receive_confirmation("Are you sure you want to clear all overrides?", true)
                        .await
                {
                    let mut rpc = MullvadProxyClient::new().await?;
                    rpc.clear_all_relay_overrides().await?;
                    println!("All overrides unset");
                }
            }
        }
        Ok(())
    }
}

fn parse_transport_port(
    port: Option<Constraint<u16>>,
    protocol: Option<Constraint<TransportProtocol>>,
    current_constraint: &Constraint<TransportPort>,
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

/// Lookup a relay among a list of [`RelayListCountry`]s by hostname.
/// The matching is exact, bar capitalization.
pub fn find_relay_by_hostname(
    countries: &[RelayListCountry],
    hostname: &str,
) -> Option<GeographicLocationConstraint> {
    countries
        .iter()
        .flat_map(|country| country.cities.clone())
        .flat_map(|city| city.relays)
        .find(|relay| relay.hostname.to_lowercase() == hostname.to_lowercase())
        .and_then(|relay| {
            relay.location.map(
                |Location {
                     country_code,
                     city_code,
                     ..
                 }| {
                    GeographicLocationConstraint::Hostname(country_code, city_code, relay.hostname)
                },
            )
        })
}

/// Return a list of all active non-bridge relays
pub async fn get_filtered_relays() -> Result<Vec<RelayListCountry>> {
    let mut rpc = MullvadProxyClient::new().await?;
    let relay_list = rpc.get_relay_locations().await?;
    Ok(filter_relays(relay_list))
}

/// Return a list of all active non-bridge relays
fn filter_relays(relay_list: RelayList) -> Vec<RelayListCountry> {
    relay_list
        .countries
        .into_iter()
        .filter_map(|mut country| {
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
                Some(country)
            } else {
                None
            }
        })
        .collect_vec()
}
