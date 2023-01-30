use crate::{location, new_rpc_client, Command, Error, Result};

use mullvad_management_interface::types;
use mullvad_types::relay_constraints::{
    BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
};
use talpid_types::net::openvpn::{self, SHADOWSOCKS_CIPHERS};

use std::{convert::TryFrom, net::SocketAddr};

pub struct Bridge;

#[mullvad_management_interface::async_trait]
impl Command for Bridge {
    fn name(&self) -> &'static str {
        "bridge"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about(
                "Manage use of bridges, socks proxies and Shadowsocks for OpenVPN. \
                Can make OpenVPN tunnels use Shadowsocks via one of the Mullvad bridge servers. \
                Can also make OpenVPN connect through any custom SOCKS5 proxy. \
                These settings also affect how the app reaches the API over Shadowsocks.",
            )
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_bridge_set_subcommand())
            .subcommand(clap::App::new("get").about("Get current bridge settings and state"))
            .subcommand(clap::App::new("list").about("List bridge relays"))
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("set", set_matches)) => Self::handle_set(set_matches).await,
            Some(("get", _)) => Self::handle_get().await,
            Some(("list", _)) => Self::list_bridge_relays().await,
            _ => unreachable!("unhandled command"),
        }
    }
}

fn create_bridge_set_subcommand() -> clap::App<'static> {
    clap::App::new("set")
        .about("Set bridge state and settings")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_set_state_subcommand())
        .subcommand(create_set_custom_settings_subcommand())
        .subcommand(
            clap::App::new("provider")
                .about(
                    "Set hosting provider(s) to select bridge relays from. The 'list' \
                        command shows the available relays and their providers.",
                )
                .arg(
                    clap::Arg::new("provider")
                        .help("The hosting provider(s) to use, or 'any' for no preference.")
                        .multiple_values(true)
                        .required(true),
                ),
        )
        .subcommand(
            clap::App::new("ownership")
                .about(
                    "Filters bridges based on ownership. The 'list' \
                       command shows the available relays and whether they're rented.",
                )
                .arg(
                    clap::Arg::new("ownership")
                        .help("Ownership preference, or 'any' for no preference.")
                        .possible_values(["any", "owned", "rented"])
                        .required(true),
                ),
        )
        .subcommand(location::get_subcommand().about(
            "Set country or city to select bridge relays from. Use the 'list' \
             command to show available alternatives.",
        ))
}

fn create_set_custom_settings_subcommand() -> clap::App<'static> {
    #[allow(unused_mut)]
    let mut local_subcommand = clap::App::new("local")
        .about("Registers a local SOCKS5 proxy")
        .arg(
            clap::Arg::new("local-port")
                .help("Specifies the port the local proxy server is listening on")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("remote-ip")
                .help("Specifies the IP of the proxy server peer")
                .required(true)
                .index(2),
        )
        .arg(
            clap::Arg::new("remote-port")
                .help("Specifies the port of the proxy server peer")
                .required(true)
                .index(3),
        );

    #[cfg(target_os = "linux")]
    {
        local_subcommand = local_subcommand.about(
            "Registers a local SOCKS5 proxy. The server must be excluded using \
           'mullvad-exclude', or `SO_MARK` must be set to '0x6d6f6c65', in order \
           to bypass firewall restrictions",
        );
    }
    #[cfg(target_os = "macos")]
    {
        local_subcommand = local_subcommand.about(
            "Registers a local SOCKS5 proxy. The server must run as root to bypass \
            firewall restrictions",
        );
    }

    clap::App::new("custom")
        .about("Configure a SOCKS5 proxy")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(local_subcommand)
        .subcommand(
            clap::App::new("remote")
                .about("Registers a remote SOCKS5 proxy")
                .arg(
                    clap::Arg::new("remote-ip")
                        .help("Specifies the IP of the remote proxy server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("remote-port")
                        .help("Specifies the port the remote proxy server is listening on")
                        .required(true)
                        .index(2),
                )
                .arg(
                    clap::Arg::new("username")
                        .help("Specifies the username for remote authentication")
                        .required(true)
                        .index(3),
                )
                .arg(
                    clap::Arg::new("password")
                        .help("Specifies the password for remote authentication")
                        .required(true)
                        .index(4),
                ),
        )
        .subcommand(
            clap::App::new("shadowsocks")
                .about("Configure bundled Shadowsocks proxy")
                .arg(
                    clap::Arg::new("remote-ip")
                        .help("Specifies the IP of the remote Shadowsocks server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("remote-port")
                        .help("Specifies the port of the remote Shadowsocks server")
                        .default_value("443")
                        .index(2),
                )
                .arg(
                    clap::Arg::new("password")
                        .help("Specifies the password on the remote Shadowsocks server")
                        .default_value("mullvad")
                        .index(3),
                )
                .arg(
                    clap::Arg::new("cipher")
                        .help("Specifies the cipher to use")
                        .default_value("aes-256-gcm")
                        .possible_values(SHADOWSOCKS_CIPHERS)
                        .index(4),
                ),
        )
}

fn create_set_state_subcommand() -> clap::App<'static> {
    clap::App::new("state").about("Set bridge state").arg(
        clap::Arg::new("policy")
            .help("Specifies whether a bridge should be used")
            .required(true)
            .index(1)
            .possible_values(["auto", "on", "off"]),
    )
}

impl Bridge {
    async fn handle_set(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("location", location_matches)) => {
                Self::handle_set_bridge_location(location_matches).await
            }
            Some(("provider", provider_matches)) => {
                Self::handle_set_bridge_provider(provider_matches).await
            }
            Some(("ownership", ownership_matches)) => {
                Self::handle_set_bridge_ownership(ownership_matches).await
            }
            Some(("custom", custom_matches)) => {
                Self::handle_bridge_set_custom_settings(custom_matches).await
            }
            Some(("state", set_matches)) => Self::handle_set_bridge_state(set_matches).await,
            _ => unreachable!("unhandled command"),
        }
    }

    async fn handle_get() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let settings = rpc.get_settings(()).await?.into_inner();
        let bridge_settings = BridgeSettings::try_from(settings.bridge_settings.unwrap()).unwrap();
        println!(
            "Bridge state: {}",
            BridgeState::try_from(settings.bridge_state.unwrap()).unwrap()
        );
        match bridge_settings {
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

    async fn handle_set_bridge_location(matches: &clap::ArgMatches) -> Result<()> {
        Self::update_bridge_settings(
            Some(location::get_constraint_from_args(matches)),
            None,
            None,
        )
        .await
    }

    async fn handle_set_bridge_provider(matches: &clap::ArgMatches) -> Result<()> {
        let providers: Vec<String> = matches.values_of_t_or_exit("provider");
        let providers = if providers.get(0).map(String::as_str) == Some("any") {
            vec![]
        } else {
            providers
        };

        Self::update_bridge_settings(None, Some(providers), None).await
    }

    async fn handle_set_bridge_ownership(matches: &clap::ArgMatches) -> Result<()> {
        let ownership =
            super::relay::parse_ownership_constraint(matches.value_of("ownership").unwrap());
        Self::update_bridge_settings(None, None, Some(ownership)).await
    }

    async fn update_bridge_settings(
        location: Option<types::RelayLocation>,
        providers: Option<Vec<String>>,
        ownership: Option<types::Ownership>,
    ) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let settings = rpc.get_settings(()).await?.into_inner();

        let bridge_settings = BridgeSettings::try_from(settings.bridge_settings.unwrap()).unwrap();
        let constraints = match bridge_settings {
            BridgeSettings::Normal(mut constraints) => {
                if let Some(new_location) = location {
                    constraints.location = Constraint::<LocationConstraint>::from(new_location);
                }
                if let Some(new_providers) = providers {
                    constraints.providers =
                        types::relay_constraints::try_providers_constraint_from_proto(
                            &new_providers,
                        )
                        .unwrap();
                }
                if let Some(new_ownership) = ownership {
                    constraints.ownership =
                        types::relay_constraints::ownership_constraint_from_proto(new_ownership);
                }
                constraints
            }
            _ => {
                let location = Constraint::<LocationConstraint>::from(location.unwrap_or_default());
                let providers = types::relay_constraints::try_providers_constraint_from_proto(
                    &providers.unwrap_or_default(),
                )
                .unwrap();
                let ownership = ownership
                    .map(types::relay_constraints::ownership_constraint_from_proto)
                    .unwrap_or_default();

                BridgeConstraints {
                    location,
                    providers,
                    ownership,
                }
            }
        };

        rpc.set_bridge_settings(
            types::BridgeSettings::try_from(BridgeSettings::Normal(constraints)).unwrap(),
        )
        .await?;
        Ok(())
    }

    async fn handle_set_bridge_state(matches: &clap::ArgMatches) -> Result<()> {
        let state = match matches.value_of("policy").unwrap() {
            "auto" => BridgeState::Auto,
            "on" => BridgeState::On,
            "off" => BridgeState::Off,
            _ => unreachable!(),
        };
        let mut rpc = new_rpc_client().await?;
        rpc.set_bridge_state(types::BridgeState::from(state))
            .await?;
        Ok(())
    }

    async fn handle_bridge_set_custom_settings(matches: &clap::ArgMatches) -> Result<()> {
        if let Some(args) = matches.subcommand_matches("local") {
            let local_port = args.value_of_t_or_exit("local-port");
            let remote_ip = args.value_of_t_or_exit("remote-ip");
            let remote_port = args.value_of_t_or_exit("remote-port");

            let local_proxy = openvpn::LocalProxySettings {
                port: local_port,
                peer: SocketAddr::new(remote_ip, remote_port),
            };
            let packed_proxy = openvpn::ProxySettings::Local(local_proxy);
            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!("{}", error);
            }

            let mut rpc = new_rpc_client().await?;
            rpc.set_bridge_settings(types::BridgeSettings::from(BridgeSettings::Custom(
                packed_proxy,
            )))
            .await?;
        } else if let Some(args) = matches.subcommand_matches("remote") {
            let remote_ip = args.value_of_t_or_exit("remote-ip");
            let remote_port = args.value_of_t_or_exit("remote-port");
            let username = args.value_of("username");
            let password = args.value_of("password");

            let auth = match (username, password) {
                (Some(username), Some(password)) => Some(openvpn::ProxyAuth {
                    username: username.to_string(),
                    password: password.to_string(),
                }),
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

            let mut rpc = new_rpc_client().await?;
            rpc.set_bridge_settings(types::BridgeSettings::from(BridgeSettings::Custom(
                packed_proxy,
            )))
            .await?;
        } else if let Some(args) = matches.subcommand_matches("shadowsocks") {
            let remote_ip = args.value_of_t_or_exit("remote-ip");
            let remote_port = args.value_of_t_or_exit("remote-port");
            let password = args.value_of_t_or_exit("password");
            let cipher = args.value_of_t_or_exit("cipher");

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

            let mut rpc = new_rpc_client().await?;
            rpc.set_bridge_settings(types::BridgeSettings::from(BridgeSettings::Custom(
                packed_proxy,
            )))
            .await?;
        } else {
            unreachable!("unhandled proxy type");
        }

        println!("proxy details have been updated");
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

    async fn list_bridge_relays() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let relay_list = rpc
            .get_relay_locations(())
            .await
            .map_err(|error| Error::RpcFailedExt("Failed to obtain relay locations", error))?
            .into_inner();

        let mut countries = Vec::new();

        for mut country in relay_list.countries {
            country.cities = country
                .cities
                .into_iter()
                .filter_map(|mut city| {
                    city.relays.retain(|relay| {
                        relay.active
                            && relay.endpoint_type == (types::relay::RelayType::Bridge as i32)
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
}
