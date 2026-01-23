use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        ObfuscationSettings, SelectedObfuscation, ShadowsocksSettings, Udp2TcpObfuscationSettings,
        WireguardPortSettings,
    },
};

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
    /// Specify which anti-censorship protocol to use, if any.
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
}

impl AntiCensorship {
    pub async fn handle(self) -> Result<()> {
        match self {
            AntiCensorship::Get => {
                let mut rpc = MullvadProxyClient::new().await?;
                let obfuscation_settings = rpc.get_settings().await?.obfuscation_settings;
                println!("mode: {}", obfuscation_settings.selected_obfuscation);
                println!("udp2tcp settings: {}", obfuscation_settings.udp2tcp);
                println!("shadowsocks settings: {}", obfuscation_settings.shadowsocks);
                println!(
                    "wireguard-port settings: {}",
                    obfuscation_settings.wireguard_port
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
                let is_valid_port = match wireguard_port.get() {
                    Constraint::Any => true,
                    Constraint::Only(port) => wireguard
                        .port_ranges
                        .into_iter()
                        .any(|range| range.contains(&port)),
                };

                if !is_valid_port {
                    return Err(anyhow::anyhow!("The specified port is invalid"));
                }
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    wireguard_port,
                    ..current_settings
                })
                .await?;
            }
        }

        println!("Updated anti-censorship settings");

        Ok(())
    }
}
