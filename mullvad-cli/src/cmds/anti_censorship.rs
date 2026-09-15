use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        LwoSettings, ObfuscationSettings, SelectedObfuscation, ShadowsocksSettings,
        Udp2TcpObfuscationSettings, WireguardPortSettings,
    },
};
use talpid_types::net::proxy::Socks5Proxy as TalpidSocks5Proxy;

use super::proxies::{Socks5LocalAdd, Socks5RemoteAdd};

#[derive(Subcommand, Debug)]
pub enum AntiCensorship {
    /// Get current anti-censorship settings
    Get,

    /// Set anti-censorship settings
    #[clap(subcommand)]
    Set(SetCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Specify which anti-censorship method to use, if any.
    Mode { mode: SelectedObfuscation },

    /// Configure udp2tcp anti-censorship.
    Udp2tcp {
        /// Port to use, or 'any'
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },

    /// Configure Shadowsocks anti-censorship.
    Shadowsocks {
        /// Port to use, or 'any'
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },

    /// Configure WireGuard port anti-censorship.
    WireguardPort {
        /// Port to use
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },

    /// Configure LWO settings.
    Lwo {
        /// Port to use
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },

    /// Relay the tunnel through a SOCKS5 proxy. Any anti-censorship method is applied on top of
    /// it, and only those that can be are allowed alongside a proxy.
    #[clap(subcommand)]
    Socks5(Socks5Proxy),
}

#[derive(Subcommand, Debug, Clone)]
pub enum Socks5Proxy {
    /// Relay through a SOCKS5 proxy running on localhost
    Local(Socks5LocalAdd),

    /// Relay through a remote SOCKS5 proxy
    Remote(Socks5RemoteAdd),

    /// Stop relaying through a SOCKS5 proxy
    Off,
}

impl AntiCensorship {
    pub async fn handle(self) -> Result<()> {
        match self {
            AntiCensorship::Get => {
                let mut rpc = MullvadProxyClient::new().await?;
                let settings = rpc.get_settings().await?;
                let obfuscation_settings = settings.obfuscation_settings;
                println!("mode: {}", obfuscation_settings.selected_obfuscation);
                println!("udp2tcp settings: {}", obfuscation_settings.udp2tcp);
                println!("shadowsocks settings: {}", obfuscation_settings.shadowsocks);
                println!(
                    "wireguard-port settings: {}",
                    obfuscation_settings.wireguard_port
                );
                println!("lwo settings: {}", obfuscation_settings.lwo);
                println!(
                    "socks5 proxy: {}",
                    match settings.tunnel_options.wireguard.socks5_proxy {
                        Some(proxy) => proxy.to_string(),
                        None => "off".to_string(),
                    }
                );
                Ok(())
            }
            AntiCensorship::Set(subcmd) => Self::set(subcmd).await,
        }
    }

    async fn set(subcmd: SetCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let current_settings = rpc.get_settings().await?.obfuscation_settings;

        match subcmd {
            SetCommands::Mode { mode } => {
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    selected_obfuscation: mode,
                    ..current_settings
                })
                .await?;
            }
            SetCommands::Udp2tcp { port } => {
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    udp2tcp: Udp2TcpObfuscationSettings { port },
                    ..current_settings
                })
                .await?;
            }
            SetCommands::Shadowsocks { port } => {
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    shadowsocks: ShadowsocksSettings { port },
                    ..current_settings
                })
                .await?;
            }
            SetCommands::WireguardPort { port } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let wireguard = rpc.get_relay_locations().await?.wireguard;
                let wireguard_port = WireguardPortSettings::from(port);
                if !is_valid_wg_port(&wireguard, port) {
                    return Err(anyhow::anyhow!("The specified port is invalid"));
                }
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    wireguard_port,
                    ..current_settings
                })
                .await?;
            }
            SetCommands::Lwo { port } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let wireguard = rpc.get_relay_locations().await?.wireguard;
                let lwo = LwoSettings { port };
                if !is_valid_wg_port(&wireguard, port) {
                    return Err(anyhow::anyhow!("The specified port is invalid"));
                }
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    lwo,
                    ..current_settings
                })
                .await?;
            }
            SetCommands::Socks5(proxy) => {
                let proxy = match proxy {
                    Socks5Proxy::Local(add) => Some(TalpidSocks5Proxy::Local(add.into())),
                    Socks5Proxy::Remote(add) => {
                        Some(TalpidSocks5Proxy::Remote(add.try_into()?))
                    }
                    Socks5Proxy::Off => None,
                };
                rpc.set_wireguard_socks5_proxy(proxy).await?;
            }
        }

        println!("Updated anti-censorship settings");

        Ok(())
    }
}

fn is_valid_wg_port(
    wireguard: &mullvad_types::relay_list::EndpointData,
    port: Constraint<u16>,
) -> bool {
    match port {
        Constraint::Any => true,
        Constraint::Only(port) => wireguard
            .port_ranges
            .iter()
            .any(|range| range.contains(&port)),
    }
}
