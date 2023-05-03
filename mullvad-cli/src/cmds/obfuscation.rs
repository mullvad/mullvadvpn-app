use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::relay_constraints::{
    Constraint, ObfuscationSettings, SelectedObfuscation, Udp2TcpObfuscationSettings,
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
    /// Specifies if obfuscation should be used with WireGuard connections.
    /// And if so, what obfuscation protocol it should use.
    Mode { mode: SelectedObfuscation },

    /// Specifies the config for the udp2tcp obfuscator.
    Udp2tcp {
        /// Port to use, or 'any'
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
        }

        println!("Updated obfuscation settings");

        Ok(())
    }
}
