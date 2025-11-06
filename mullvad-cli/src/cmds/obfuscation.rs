use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        ObfuscationSettings, SelectedObfuscation, ShadowsocksSettings, Udp2TcpObfuscationSettings,
        WireguardPortSetting,
    },
};

#[derive(Subcommand, Debug)]
pub enum Obfuscation {
    /// Get current obfuscation settings
    Get,

    /// Set obfuscation settings
    #[clap(subcommand)]
    Set(SetCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum SetCommands {
    /// Specify which obfuscation protocol to use, if any.
    Mode { mode: SelectedObfuscation },

    /// Configure udp2tcp obfuscation.
    Udp2tcp {
        /// Port to use, or 'any'
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },

    /// Configure Shadowsocks obfuscation.
    Shadowsocks {
        /// Port to use, or 'any'
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },
    Port {
        /// Port to use
        #[arg(long, short = 'p')]
        port: Constraint<u16>,
    },
}

impl Obfuscation {
    pub async fn handle(self) -> Result<()> {
        match self {
            Obfuscation::Get => {
                let mut rpc = MullvadProxyClient::new().await?;
                let obfuscation_settings = rpc.get_settings().await?.obfuscation_settings;
                println!(
                    "Obfuscation mode: {}",
                    obfuscation_settings.selected_obfuscation
                );
                println!("udp2tcp settings: {}", obfuscation_settings.udp2tcp);
                println!("Shadowsocks settings: {}", obfuscation_settings.shadowsocks);
                Ok(())
            }
            Obfuscation::Set(subcmd) => Self::set(subcmd).await,
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
            SetCommands::Port { port } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let wireguard = rpc.get_relay_locations().await?.wireguard;
                let port = WireguardPortSetting::from(port);

                let is_valid_port = if let Some(port) = port.number() {
                    wireguard
                        .port_ranges
                        .into_iter()
                        .any(|range| range.contains(&port))
                } else {
                    true
                };

                if !is_valid_port {
                    return Err(anyhow::anyhow!("The specified port is invalid"));
                }
                rpc.set_obfuscation_settings(ObfuscationSettings {
                    port,
                    ..current_settings
                })
                .await?;
            }
        }

        println!("Updated obfuscation settings");

        Ok(())
    }
}
