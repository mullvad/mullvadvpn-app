use crate::{location, Command, Error, MullvadProxyClient, Result};
use itertools::Itertools;
use std::{
    io::{self, BufRead},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use mullvad_types::{
    relay_constraints::{
        Constraint, LocationConstraint, Match, OpenVpnConstraints, Ownership, Providers,
        RelayConstraintsUpdate, RelaySettings, RelaySettingsUpdate, TransportPort,
        WireguardConstraints,
    },
    relay_list::{RelayEndpointData, RelayListCountry},
    ConnectionConfig, CustomTunnelEndpoint,
};
use talpid_types::net::{
    all_of_the_internet, openvpn, wireguard, Endpoint, IpVersion, TransportProtocol, TunnelType,
};

pub struct Relay;

#[mullvad_management_interface::async_trait]
impl Command for Relay {
    fn name(&self) -> &'static str {
        "relay"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Manage relay and tunnel constraints")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::App::new("set")
                    .about(
                        "Set relay server selection parameters. Such as location and port/protocol",
                    )
                    .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(
                        clap::App::new("custom")
                            .about("Set a custom VPN relay")
                            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(clap::App::new("wireguard")
                                .arg(
                                    clap::Arg::new("host")
                                        .help("Hostname or IP")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("port")
                                        .help("Remote network port")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("peer-pubkey")
                                        .help("Base64 encoded peer public key")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("v4-gateway")
                                        .help("IPv4 gateway address")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("addr")
                                        .help("Local address of wireguard tunnel")
                                        .required(true)
                                        .multiple_values(true),
                                )
                                .arg(
                                    clap::Arg::new("v6-gateway")
                                        .help("IPv6 gateway address")
                                        .long("v6-gateway")
                                        .takes_value(true),
                                )
                            )
                            .subcommand(clap::App::new("openvpn")
                                .arg(
                                    clap::Arg::new("host")
                                        .help("Hostname or IP")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("port")
                                        .help("Remote network port")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("username")
                                        .help("Username to be used with the OpenVpn relay")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("password")
                                        .help("Password to be used with the OpenVpn relay")
                                        .required(true),
                                )
                                .arg(
                                    clap::Arg::new("protocol")
                                        .help("Transport protocol")
                                        .long("protocol")
                                        .default_value("udp")
                                        .possible_values(["udp", "tcp"]),
                                )
                            )
                    )
                    .subcommand(
                        location::get_subcommand()
                            .about("Set country or city to select relays from. Use the 'list' \
                                   command to show available alternatives.")
                    )
                    .subcommand(
                        clap::App::new("hostname")
                            .about("Set the exact relay to use via its hostname. Shortcut for \
                                'location <country> <city> <hostname>'.")
                            .arg(
                                clap::Arg::new("hostname")
                                    .help("The hostname")
                                    .required(true),
                            ),
                    )
                    .subcommand(
                        clap::App::new("provider")
                            .about("Set hosting provider(s) to select relays from. The 'list' \
                                   command shows the available relays and their providers.")
                            .arg(
                                clap::Arg::new("provider")
                                .help("The hosting provider(s) to use, or 'any' for no preference.")
                                .multiple_values(true)
                                .required(true)
                            )
                    )
                    .subcommand(
                        clap::App::new("ownership")
                            .about("Filters relays based on ownership. The 'list' \
                                   command shows the available relays and whether they're rented.")
                            .arg(
                                clap::Arg::new("ownership")
                                .help("Ownership preference, or 'any' for no preference.")
                                .possible_values(["any", "owned", "rented"])
                                .required(true)
                            )
                    )
                    .subcommand(
                        clap::App::new("tunnel")
                            .about("Set tunnel protocol-specific constraints.")
                            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(
                                clap::App::new("openvpn")
                                    .about("Set OpenVPN-specific constraints")
                                    .setting(clap::AppSettings::ArgRequiredElseHelp)
                                    .arg(
                                        clap::Arg::new("port")
                                            .help("Port to use. Either 'any' or a specific port")
                                            .long("port")
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::new("transport protocol")
                                            .help("Transport protocol")
                                            .long("protocol")
                                            .possible_values(["any", "udp", "tcp"])
                                            .takes_value(true),
                                    )
                            )
                            .subcommand(
                                clap::App::new("wireguard")
                                    .about("Set WireGuard-specific constraints")
                                    .setting(clap::AppSettings::ArgRequiredElseHelp)
                                    .arg(
                                        clap::Arg::new("port")
                                            .help("Port to use. Either 'any' or a specific port")
                                            .long("port")
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::new("ip version")
                                            .long("ipv")
                                            .possible_values(["any", "4", "6"])
                                            .takes_value(true),
                                    )
                                    .arg(
                                        clap::Arg::new("entry location")
                                            .help("Entry endpoint to use. This can be 'any', 'none', or \
                                                   any location that is valid with 'set location', \
                                                   such as 'se got'.")
                                            .long("entry-location")
                                            .min_values(1)
                                            .max_values(3),
                                    )
                            )
                    )
                    .subcommand(clap::App::new("tunnel-protocol")
                                .about("Set tunnel protocol")
                                .arg(
                                    clap::Arg::new("tunnel protocol")
                                    .required(true)
                                    .index(1)
                                    .possible_values(["any", "wireguard", "openvpn", ]),
                                    )
                                ),
            )
            .subcommand(clap::App::new("get"))
            .subcommand(
                clap::App::new("list").about("List available countries and cities"),
            )
            .subcommand(
                clap::App::new("update")
                    .about("Update the list of available countries and cities"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            self.set(set_matches).await
        } else if matches.subcommand_matches("get").is_some() {
            self.get().await
        } else if matches.subcommand_matches("list").is_some() {
            self.list().await
        } else if matches.subcommand_matches("update").is_some() {
            self.update().await
        } else {
            unreachable!("No relay command given");
        }
    }
}

impl Relay {
    async fn update_constraints(&self, update: RelaySettingsUpdate) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.update_relay_settings(update).await?;
        println!("Relay constraints updated");
        Ok(())
    }

    async fn set(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(custom_matches) = matches.subcommand_matches("custom") {
            self.set_custom(custom_matches).await
        } else if let Some(location_matches) = matches.subcommand_matches("location") {
            self.set_location(location_matches).await
        } else if let Some(relay_matches) = matches.subcommand_matches("hostname") {
            self.set_hostname(relay_matches).await
        } else if let Some(providers_matches) = matches.subcommand_matches("provider") {
            self.set_providers(providers_matches).await
        } else if let Some(ownership_matches) = matches.subcommand_matches("ownership") {
            self.set_ownership(ownership_matches).await
        } else if let Some(matches) = matches.subcommand_matches("tunnel") {
            if let Some(tunnel_matches) = matches.subcommand_matches("openvpn") {
                self.set_openvpn_constraints(tunnel_matches).await
            } else if let Some(tunnel_matches) = matches.subcommand_matches("wireguard") {
                self.set_wireguard_constraints(tunnel_matches).await
            } else {
                unreachable!("Invalid tunnel protocol");
            }
        } else if let Some(tunnel_matches) = matches.subcommand_matches("tunnel-protocol") {
            self.set_tunnel_protocol(tunnel_matches).await
        } else {
            unreachable!("No set relay command given");
        }
    }

    async fn set_custom(&self, matches: &clap::ArgMatches) -> Result<()> {
        let custom_endpoint = match matches.subcommand() {
            Some(("openvpn", openvpn_matches)) => Self::read_custom_openvpn_relay(openvpn_matches),
            Some(("wireguard", wg_matches)) => Self::read_custom_wireguard_relay(wg_matches)?,
            _ => unreachable!("No set relay command given"),
        };
        self.update_constraints(RelaySettingsUpdate::CustomTunnelEndpoint(custom_endpoint))
            .await
    }

    fn read_custom_openvpn_relay(matches: &clap::ArgMatches) -> CustomTunnelEndpoint {
        let host = matches.value_of_t_or_exit("host");
        let port = matches.value_of_t_or_exit("port");
        let username = matches.value_of_t_or_exit("username");
        let password = matches.value_of_t_or_exit("password");
        let protocol: String = matches.value_of_t_or_exit("protocol");

        let protocol = Self::validate_transport_protocol(&protocol);

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

    fn read_custom_wireguard_relay(matches: &clap::ArgMatches) -> Result<CustomTunnelEndpoint> {
        let host = matches.value_of_t_or_exit("host");
        let port = matches.value_of_t_or_exit("port");
        let addresses: Vec<IpAddr> = matches.values_of_t_or_exit("addr");
        let peer_key_str: String = matches.value_of_t_or_exit("peer-pubkey");
        let ipv4_gateway: Ipv4Addr = matches.value_of_t_or_exit("v4-gateway");
        let ipv6_gateway = match matches.value_of_t::<Ipv6Addr>("v6-gateway") {
            Ok(gateway) => Some(gateway),
            Err(e) => match e.kind {
                clap::ErrorKind::ArgumentNotFound => None,
                _ => e.exit(),
            },
        };
        let mut private_key_str = String::new();
        println!("Reading private key from standard input");
        let _ = io::stdin().lock().read_line(&mut private_key_str);
        if private_key_str.trim().is_empty() {
            eprintln!("Expected to read private key from standard input");
        }
        let peer_public_key = wireguard::PublicKey::from_base64(&peer_key_str)
            .map_err(|_| Error::InvalidCommand("invalid public key"))?;
        let private_key = wireguard::PrivateKey::from_base64(&private_key_str)
            .map_err(|_| Error::InvalidCommand("invalid private key"))?;

        Ok(CustomTunnelEndpoint {
            host,
            config: ConnectionConfig::Wireguard(wireguard::ConnectionConfig {
                tunnel: wireguard::TunnelConfig {
                    private_key,
                    addresses,
                },
                peer: wireguard::PeerConfig {
                    public_key: peer_public_key,
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

    fn validate_transport_protocol(protocol: &str) -> TransportProtocol {
        match protocol {
            "udp" => TransportProtocol::Udp,
            "tcp" => TransportProtocol::Tcp,
            _ => clap::Error::raw(
                clap::ErrorKind::ValueValidation,
                "invalid transport protocol",
            )
            .exit(),
        }
    }

    async fn set_hostname(&self, matches: &clap::ArgMatches) -> Result<()> {
        let hostname = matches.value_of("hostname").unwrap();
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

        if let Some(location) = find_relay() {
            println!("Setting location constraint to {location}");
            self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
                location: Some(Constraint::Only(location)),
                ..Default::default()
            }))
            .await
        } else {
            clap::Error::raw(clap::ErrorKind::ValueValidation, "No matching server found").exit()
        }
    }

    async fn set_location(&self, matches: &clap::ArgMatches) -> Result<()> {
        let location_constraint = location::get_constraint_from_args(matches);
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
        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(location_constraint),
            ..Default::default()
        }))
        .await
    }

    async fn set_providers(&self, matches: &clap::ArgMatches) -> Result<()> {
        let providers: Vec<String> = matches.values_of_t_or_exit("provider");
        let providers = if providers.get(0).map(String::as_str) == Some("any") {
            Constraint::Any
        } else {
            Constraint::Only(Providers::new(providers.into_iter()).unwrap())
        };

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            providers: Some(providers),
            ..Default::default()
        }))
        .await
    }

    async fn set_ownership(&self, matches: &clap::ArgMatches) -> Result<()> {
        let ownership = parse_ownership_constraint(matches.value_of("ownership").unwrap());

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            ownership: Some(ownership),
            ..Default::default()
        }))
        .await
    }

    async fn set_openvpn_constraints(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut openvpn_constraints = {
            let mut rpc = MullvadProxyClient::new().await?;
            Self::get_openvpn_constraints(&mut rpc).await?
        };
        openvpn_constraints.port = parse_transport_port(matches, &mut openvpn_constraints.port)?;

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
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

    async fn set_wireguard_constraints(&self, matches: &clap::ArgMatches) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let wireguard = rpc.get_relay_locations().await?.wireguard;
        let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

        if let Some(port) = matches.value_of("port") {
            wireguard_constraints.port = match parse_port_constraint(port)? {
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

        if let Some(ipv) = matches.value_of("ip version") {
            wireguard_constraints.ip_version = parse_ip_version_constraint(ipv);
        }
        if let Some(entry) = matches.values_of("entry location") {
            match parse_entry_location_constraint(entry) {
                Some(location) => {
                    wireguard_constraints.entry_location = location;
                    wireguard_constraints.use_multihop = true;
                }
                None => {
                    wireguard_constraints.use_multihop = false;
                }
            }
        }

        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
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

    async fn set_tunnel_protocol(&self, matches: &clap::ArgMatches) -> Result<()> {
        let tunnel_protocol = match matches.value_of("tunnel protocol").unwrap() {
            "wireguard" => Constraint::Only(TunnelType::Wireguard),
            "openvpn" => Constraint::Only(TunnelType::OpenVpn),
            "any" => Constraint::Any,
            _ => unreachable!(),
        };
        self.update_constraints(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            tunnel_protocol: Some(tunnel_protocol),
            ..Default::default()
        }))
        .await
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_settings = rpc.get_settings().await?.relay_settings;
        println!("Current constraints: {relay_settings}");
        Ok(())
    }

    async fn list(&self) -> Result<()> {
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

    async fn update(&self) -> Result<()> {
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
}

fn parse_port_constraint(raw_port: &str) -> Result<Constraint<u16>> {
    match raw_port.to_lowercase().as_str() {
        "any" => Ok(Constraint::Any),
        port => Ok(Constraint::Only(u16::from_str(port).map_err(|_| {
            Error::InvalidCommand("Invalid port. Must be \"any\" or 0-65535.")
        })?)),
    }
}

fn parse_protocol(raw_protocol: &str) -> Constraint<TransportProtocol> {
    match raw_protocol {
        "any" => Constraint::Any,
        "udp" => Constraint::Only(TransportProtocol::Udp),
        "tcp" => Constraint::Only(TransportProtocol::Tcp),
        _ => unreachable!(),
    }
}

fn parse_ip_version_constraint(raw_protocol: &str) -> Constraint<IpVersion> {
    match raw_protocol {
        "any" => Constraint::Any,
        "4" => Constraint::Only(IpVersion::V4),
        "6" => Constraint::Only(IpVersion::V6),
        _ => unreachable!(),
    }
}

fn parse_entry_location_constraint<'a, T: Iterator<Item = &'a str>>(
    mut location: T,
) -> Option<Constraint<LocationConstraint>> {
    let country = location.next().unwrap();

    if country == "none" {
        return None;
    }

    Some(location::get_constraint(
        country,
        location.next(),
        location.next(),
    ))
}

fn parse_transport_port(
    matches: &clap::ArgMatches,
    current_constraint: &mut Constraint<TransportPort>,
) -> Result<Constraint<TransportPort>> {
    let protocol = match matches.value_of("transport protocol") {
        Some(protocol) => parse_protocol(protocol),
        None => {
            if let Constraint::Only(ref transport_port) = current_constraint {
                Constraint::Only(transport_port.protocol)
            } else {
                Constraint::Any
            }
        }
    };
    let mut port = match matches.value_of("port") {
        Some(port) => parse_port_constraint(port)?,
        None => current_constraint
            .map(|port| port.port)
            .unwrap_or(Constraint::Any),
    };
    if port.is_only() && protocol.is_any() && !matches.is_present("port") {
        // Reset the port if the transport protocol is set to any.
        println!("The port constraint was set to 'any'");
        port = Constraint::Any;
    }
    match (port, protocol) {
        (Constraint::Any, Constraint::Any) => Ok(Constraint::Any),
        (Constraint::Any, Constraint::Only(protocol)) => Ok(Constraint::Only(TransportPort {
            protocol,
            port: Constraint::Any,
        })),
        (Constraint::Only(port), Constraint::Only(protocol)) => {
            Ok(Constraint::Only(TransportPort {
                protocol,
                port: Constraint::Only(port),
            }))
        }
        (Constraint::Only(_), Constraint::Any) => Err(Error::InvalidCommand(
            "a transport protocol must be given to select a specific port",
        )),
    }
}

pub fn parse_ownership_constraint(constraint: &str) -> Constraint<Ownership> {
    match constraint {
        "any" => Constraint::Any,
        "owned" => Constraint::Only(Ownership::MullvadOwned),
        "rented" => Constraint::Only(Ownership::Rented),
        _ => unreachable!(),
    }
}
