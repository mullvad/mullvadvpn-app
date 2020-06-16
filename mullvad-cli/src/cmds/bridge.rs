use crate::{location, new_rpc_client, Command, Result};
use clap::value_t;

use mullvad_types::relay_constraints::{BridgeConstraints, BridgeSettings, BridgeState};
use talpid_types::net::openvpn::{self, SHADOWSOCKS_CIPHERS};

use std::net::{IpAddr, SocketAddr};

pub struct Bridge;

#[async_trait::async_trait]
impl Command for Bridge {
    fn name(&self) -> &'static str {
        "bridge"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage use of bridges")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_bridge_set_subcommand())
            .subcommand(
                clap::SubCommand::with_name("get").about("Get current bridge settings and state"),
            )
            .subcommand(clap::SubCommand::with_name("list").about("List bridge relays"))
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("set", Some(set_matches)) => Self::handle_set(set_matches),
            ("get", _) => Self::handle_get(),
            ("list", _) => Self::list_bridge_relays(),
            _ => unreachable!("unhandled command"),
        }
    }
}

fn create_bridge_set_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("set")
        .about("Set bridge state and settings")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create_set_state_subcommand())
        .subcommand(create_set_custom_settings_subcommand())
        .subcommand(location::get_subcommand().about(
            "Set country or city to select bridge relays from. Use the 'list' \
             command to show available alternatives.",
        ))
}


fn create_set_custom_settings_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("custom")
        .about("Configure a SOCKS5 proxy")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::SubCommand::with_name("local")
                .about("Registers a local SOCKS5 proxy")
                .arg(
                    clap::Arg::with_name("local-port")
                        .help("Specifies the port the local proxy server is listening on")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the proxy server peer")
                        .required(true)
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port of the proxy server peer")
                        .required(true)
                        .index(3),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("remote")
                .about("Registers a remote SOCKS5 proxy")
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the remote proxy server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port the remote proxy server is listening on")
                        .required(true)
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("username")
                        .help("Specifies the username for remote authentication")
                        .required(true)
                        .index(3),
                )
                .arg(
                    clap::Arg::with_name("password")
                        .help("Specifies the password for remote authentication")
                        .required(true)
                        .index(4),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("shadowsocks")
                .about("Configure bundled Shadowsocks proxy")
                .arg(
                    clap::Arg::with_name("remote-ip")
                        .help("Specifies the IP of the remote Shadowsocks server")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::with_name("remote-port")
                        .help("Specifies the port of the remote Shadowsocks server")
                        .default_value("443")
                        .index(2),
                )
                .arg(
                    clap::Arg::with_name("password")
                        .help("Specifies the password on the remote Shadowsocks server")
                        .default_value("23#dfsbbb")
                        .index(3),
                )
                .arg(
                    clap::Arg::with_name("cipher")
                        .help("Specifies the cipher to use")
                        .default_value("chacha20")
                        .possible_values(SHADOWSOCKS_CIPHERS)
                        .index(4),
                ),
        )
}

fn create_set_state_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("state")
        .about("Set bridge state")
        .arg(
            clap::Arg::with_name("state")
                .help("Specifies whether a bridge should be used")
                .required(true)
                .index(1)
                .possible_values(&["auto", "on", "off"]),
        )
}

impl Bridge {
    fn handle_set(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("location", Some(location_matches)) => {
                Self::handle_set_bridge_location(location_matches)
            }
            ("custom", Some(custom_matches)) => {
                Self::handle_bridge_set_custom_settings(custom_matches)
            }
            ("state", Some(set_matches)) => Self::handle_set_bridge_state(set_matches),
            _ => unreachable!("unhandled command"),
        }
    }

    fn handle_get() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let settings = rpc.get_settings()?;
        println!("Bridge state - {}", settings.get_bridge_state());
        match settings.bridge_settings {
            BridgeSettings::Custom(proxy) => {
                match proxy {
                    openvpn::ProxySettings::Local(local_proxy) => {
                        Self::print_local_proxy(&local_proxy)
                    }
                    openvpn::ProxySettings::Remote(remote_proxy) => {
                        Self::print_remote_proxy(&remote_proxy)
                    }
                    openvpn::ProxySettings::Shadowsocks(shadowsocks_proxy) => {
                        Self::print_shadowsocks_proxy(&shadowsocks_proxy)
                    }
                };
            }
            BridgeSettings::Normal(constraints) => {
                println!("Bridge constraints: {}", constraints);
            }
        };
        Ok(())
    }

    fn handle_set_bridge_location(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let location = location::get_constraint(matches);
        let mut rpc = new_rpc_client()?;
        rpc.set_bridge_settings(BridgeSettings::Normal(BridgeConstraints { location }))?;
        Ok(())
    }

    fn handle_set_bridge_state(matches: &clap::ArgMatches<'_>) -> Result<()> {
        let state = match matches.value_of("state").unwrap() {
            "auto" => BridgeState::Auto,
            "on" => BridgeState::On,
            "off" => BridgeState::Off,
            _ => unreachable!(),
        };
        let mut rpc = new_rpc_client()?;
        rpc.set_bridge_state(state)?;
        Ok(())
    }

    fn handle_bridge_set_custom_settings(matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(args) = matches.subcommand_matches("local") {
            let local_port =
                value_t!(args.value_of("local-port"), u16).unwrap_or_else(|e| e.exit());
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());

            let proxy = openvpn::LocalProxySettings {
                port: local_port,
                peer: SocketAddr::new(remote_ip, remote_port),
            };

            let packed_proxy = openvpn::ProxySettings::Local(proxy);

            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else if let Some(args) = matches.subcommand_matches("remote") {
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());
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
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else if let Some(args) = matches.subcommand_matches("shadowsocks") {
            let remote_ip =
                value_t!(args.value_of("remote-ip"), IpAddr).unwrap_or_else(|e| e.exit());
            let remote_port =
                value_t!(args.value_of("remote-port"), u16).unwrap_or_else(|e| e.exit());
            let password = args.value_of("password").unwrap().to_string();
            let cipher = args.value_of("cipher").unwrap().to_string();

            let proxy = openvpn::ShadowsocksProxySettings {
                peer: SocketAddr::new(remote_ip, remote_port),
                password,
                cipher,
            };

            let packed_proxy = openvpn::ProxySettings::Shadowsocks(proxy);

            if let Err(error) = openvpn::validate_proxy_settings(&packed_proxy) {
                panic!(error);
            }

            let mut rpc = new_rpc_client()?;
            rpc.set_bridge_settings(BridgeSettings::Custom(packed_proxy))?;
        } else {
            unreachable!("unhandled proxy type");
        }

        println!("proxy details have been updated");
        Ok(())
    }

    fn print_local_proxy(proxy: &openvpn::LocalProxySettings) {
        println!("proxy: local");
        println!("  local port: {}", proxy.port);
        println!("  peer IP: {}", proxy.peer.ip());
        println!("  peer port: {}", proxy.peer.port());
    }

    fn print_remote_proxy(proxy: &openvpn::RemoteProxySettings) {
        println!("proxy: remote");
        println!("  server IP: {}", proxy.address.ip());
        println!("  server port: {}", proxy.address.port());

        if let Some(ref auth) = proxy.auth {
            println!("  auth username: {}", auth.username);
            println!("  auth password: {}", auth.password);
        } else {
            println!("  auth: none");
        }
    }

    fn print_shadowsocks_proxy(proxy: &openvpn::ShadowsocksProxySettings) {
        println!("proxy: Shadowsocks");
        println!("  peer IP: {}", proxy.peer.ip());
        println!("  peer port: {}", proxy.peer.port());
        println!("  password: {}", proxy.password);
        println!("  cipher: {}", proxy.cipher);
    }

    fn list_bridge_relays() -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let mut locations = rpc.get_relay_locations()?;

        locations.countries = locations
            .countries
            .into_iter()
            .filter_map(|mut country| {
                country.cities = country
                    .cities
                    .into_iter()
                    .filter_map(|mut city| {
                        city.relays
                            .retain(|relay| relay.active && !relay.bridges.is_empty());
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
            .collect();

        locations
            .countries
            .sort_by(|c1, c2| natord::compare_ignore_case(&c1.name, &c2.name));
        for mut country in locations.countries {
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
                    println!("\t\t{} ({})", relay.hostname, relay.ipv4_addr_in);
                }
            }
            println!();
        }
        Ok(())
    }
}
