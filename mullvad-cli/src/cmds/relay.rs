use anyhow::{Context, Result, anyhow, bail};
use clap::Subcommand;
use itertools::Itertools;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    ConnectionConfig, CustomTunnelEndpoint,
    constraints::{Constraint, Match},
    location::CountryCode,
    relay_constraints::{
        GeographicLocationConstraint, LocationConstraint, LocationConstraintFormatter, Ownership,
        Provider, Providers, RelayConstraints, RelayOverride, RelaySettings, WireguardConstraints,
        allowed_ip::AllowedIps,
    },
    relay_list::{RelayEndpointData, RelayListCountry},
};
use std::{
    collections::HashMap,
    io::BufRead,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use talpid_types::net::{IpVersion, wireguard};

use super::{BooleanOption, relay_constraints::LocationArgs};
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

    /// Set tunnel port constraint
    Port {
        /// Port to use, or 'any'
        port: Constraint<u16>,
    },

    /// Set tunnel IP version constraint
    IpVersion {
        /// IP protocol to use, or 'any'
        ip_version: Constraint<IpVersion>,
    },

    /// Enable or disable multihop
    Multihop {
        /// Whether to enable multihop. The location constraints are specified with
        /// 'entry-location'.
        use_multihop: BooleanOption,
    },

    /// Set entry location constraints for multihop
    #[clap(subcommand)]
    Entry(EntryArgs),

    /// Set a custom WireGuard relay
    Custom {
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
pub enum EntryArgs {
    /// Location of entry relay. This can be 'any' or any location that is valid with 'set
    /// location', such as 'se got'.
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
    Location(LocationArgs),
    /// Name of custom list to use to pick entry endpoint.
    CustomList { custom_list_name: String },
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

                print_option!("Provider(s)", constraints.providers,);
                print_option!("Ownership", constraints.ownership,);

                println!("WireGuard constraints");

                print_option!("Port", constraints.wireguard_constraints.port,);

                print_option!("IP protocol", constraints.wireguard_constraints.ip_version,);

                print_option!(
                    "Multihop state",
                    if constraints.wireguard_constraints.multihop() {
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
        let mut countries = get_active_relays().await?;
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
            SetCommands::Location(location) => Self::set_location(location).await,
            SetCommands::CustomList { custom_list_name } => {
                Self::set_custom_list(custom_list_name).await
            }
            SetCommands::Provider { providers } => Self::set_providers(providers).await,
            SetCommands::Ownership { ownership } => Self::set_ownership(ownership).await,
            SetCommands::Port { port } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let wireguard = rpc.get_relay_locations().await?.wireguard;
                let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

                wireguard_constraints.port = match port {
                    Constraint::Any => Constraint::Any,
                    Constraint::Only(specific_port) => {
                        let is_valid_port = wireguard
                            .port_ranges
                            .into_iter()
                            .any(|range| range.contains(&specific_port));
                        if !is_valid_port {
                            return Err(anyhow!("The specified port is invalid"));
                        }
                        Constraint::Only(specific_port)
                    }
                };

                Self::update_constraints(|constraints| {
                    constraints.wireguard_constraints = wireguard_constraints;
                })
                .await
            }
            SetCommands::IpVersion { ip_version } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

                wireguard_constraints.ip_version = ip_version;

                Self::update_constraints(|constraints| {
                    constraints.wireguard_constraints = wireguard_constraints;
                })
                .await
            }
            SetCommands::Multihop { use_multihop } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

                wireguard_constraints.use_multihop(*use_multihop);

                Self::update_constraints(|constraints| {
                    constraints.wireguard_constraints = wireguard_constraints;
                })
                .await
            }
            SetCommands::Entry(entry_args) => {
                let mut rpc = MullvadProxyClient::new().await?;
                let mut wireguard_constraints = Self::get_wireguard_constraints(&mut rpc).await?;

                match entry_args {
                    EntryArgs::Location(location_args) => {
                        let relay_filter = |relay: &mullvad_types::relay_list::Relay| {
                            relay.active
                                && matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
                        };
                        let location_constraint =
                            resolve_location_constraint(&mut rpc, location_args, relay_filter)
                                .await?;

                        wireguard_constraints.entry_location =
                            location_constraint.map(LocationConstraint::from);
                    }
                    EntryArgs::CustomList { custom_list_name } => {
                        let list_id =
                            super::custom_list::find_list_by_name(&mut rpc, &custom_list_name)
                                .await?
                                .id();
                        wireguard_constraints.entry_location =
                            Constraint::Only(LocationConstraint::CustomList { list_id });
                    }
                }

                Self::update_constraints(|constraints| {
                    constraints.wireguard_constraints = wireguard_constraints;
                })
                .await
            }
            SetCommands::Custom {
                host,
                port,
                peer_pubkey,
                tunnel_ip,
                v4_gateway,
                v6_gateway,
            } => {
                let custom_endpoint = Self::read_custom_wireguard_relay(
                    host,
                    port,
                    peer_pubkey,
                    tunnel_ip,
                    v4_gateway,
                    v6_gateway,
                )
                .await?;
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_relay_settings(RelaySettings::CustomTunnelEndpoint(custom_endpoint))
                    .await?;
                println!("Relay constraints updated");
                Ok(())
            }
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
                    allowed_ips: AllowedIps::allow_all().resolve(Some(ipv4_gateway), ipv6_gateway),
                    endpoint: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port),
                    psk: None,
                    constant_packet_size: false,
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
        let mut rpc = MullvadProxyClient::new().await?;
        let relay_settings = rpc.get_settings().await?.get_relay_settings();
        if let RelaySettings::CustomTunnelEndpoint(_custom) = relay_settings {
            bail!("Cannot change location while custom endpoint is set");
        }

        let location_constraint =
            resolve_location_constraint(&mut rpc, location_constraint_args, |relay| {
                relay.active && matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
            })
            .await?;

        Self::update_constraints(|constraints| {
            constraints.location = location_constraint.map(LocationConstraint::from);
        })
        .await
    }

    async fn set_custom_list(custom_list_name: String) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let list_id = super::custom_list::find_list_by_name(&mut rpc, &custom_list_name)
            .await?
            .id();
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

    async fn update_override(
        hostname: &str,
        update_fn: impl FnOnce(&mut RelayOverride),
        warn_non_existent_hostname: bool,
    ) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let settings = rpc.get_settings().await?;

        if warn_non_existent_hostname {
            let relay_list = rpc.get_relay_locations().await?;
            if !relay_list.relays().any(|relay| {
                relay.active
                    && relay.endpoint_data != RelayEndpointData::Bridge
                    && relay.hostname.to_lowercase() == hostname.to_lowercase()
            }) {
                eprintln!("Warning: Setting overrides for an unrecognized server");
            };
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
                for country in get_active_relays().await? {
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

fn relay_to_geographical_constraint(
    relay: mullvad_types::relay_list::Relay,
) -> GeographicLocationConstraint {
    GeographicLocationConstraint::Hostname(
        relay.location.country_code,
        relay.location.city_code,
        relay.hostname,
    )
}

/// Parses the [`LocationArgs`] into a [`Constraint<GeographicLocationConstraint>`].
///
/// See the documentation of [`mullvad relay set location`](SetCommands) for a description
/// of what arguments are valid.
///
/// Usually, only a subset of relays are relevant, e.g. only active server of a certain type.
/// Use `relay_filter` to pass in this requirement. If the user gives a host not matching the
/// filter an appropriate error is given.
pub async fn resolve_location_constraint(
    rpc: &mut MullvadProxyClient,
    location_constraint_args: LocationArgs,
    relay_filter: impl FnOnce(&mullvad_types::relay_list::Relay) -> bool,
) -> Result<Constraint<GeographicLocationConstraint>> {
    let relay_iter = rpc.get_relay_locations().await?.into_relays();
    match relay_iter
        .clone()
        .find(|relay| relay.hostname.to_lowercase() == location_constraint_args.country)
    {
        Some(matching_relay) => {
            if relay_filter(&matching_relay) {
                Ok(Constraint::Only(relay_to_geographical_constraint(
                    matching_relay,
                )))
            } else {
                bail!(
                    "The relay `{}` is not valid for this operation",
                    location_constraint_args.country
                )
            }
        }
        _ => {
            // The Constraint was not a relay, assuming it to be a location
            let location_constraint: Constraint<GeographicLocationConstraint> =
                Constraint::from(location_constraint_args);

            // If the location constraint was not "any", then validate the country/city
            if let Constraint::Only(constraint) = &location_constraint {
                let found = relay_iter.clone().any(|relay| constraint.matches(&relay));

                if !found {
                    bail!("Invalid location argument");
                }
            }

            Ok(location_constraint)
        }
    }
}

/// Return a list of all relays that are active and not bridges
pub async fn get_active_relays() -> Result<Vec<RelayListCountry>> {
    let mut rpc = MullvadProxyClient::new().await?;
    let relay_list = rpc.get_relay_locations().await?;
    Ok(relay_list
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
        .collect_vec())
}
