use anyhow::{Result, bail};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{RelayConstraints, RelaySettings},
};

#[derive(clap::Subcommand, Debug)]
pub enum DebugCommands {
    /// Block all internet connection by setting an invalid relay constraint.
    BlockConnection,
    /// Relay
    #[clap(subcommand)]
    Relay(RelayDebugCommands),
    /// Handy commands for interacting with the app release rollout system.
    #[clap(subcommand)]
    Rollout(RolloutDebugCommands),
}

#[derive(clap::Subcommand, Debug)]
pub enum RelayDebugCommands {
    /// Inactivate this _category of relays_ - a category can be one of the following: a relay, a
    /// city, a country or a tunnel protocol (`openvpn` or `wireguard`).
    Disable { relay: String },
    /// (Re)Activate this _category of relays_ - a category can be one of the following: a relay, a
    /// city, a country or a tunnel protocol (`openvpn` or `wireguard`).
    Enable { relay: String },
}

#[derive(clap::Subcommand, Debug)]
pub enum RolloutDebugCommands {
    /// Print your rollout threshold.
    Get,
    /// Generate a new rollout threshold (overwrites the current threshold value)
    Reroll,
    /// Set your rollout threshold seed to a known value.
    ///
    /// The seed is used to generate a rollout threshold.
    Seed { value: u32 },
}

impl DebugCommands {
    pub async fn handle(self) -> Result<()> {
        match self {
            DebugCommands::BlockConnection => {
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
                constraints.location = Constraint::Only(
                    mullvad_types::relay_constraints::LocationConstraint::Location(
                        mullvad_types::relay_constraints::GeographicLocationConstraint::Country(
                            "xx".into(),
                        ),
                    ),
                );
                rpc.set_relay_settings(RelaySettings::Normal(constraints))
                    .await?;

                rpc.connect_tunnel().await?;

                eprintln!("WARNING: ENTERED BLOCKED MODE");
                Ok(())
            }
            DebugCommands::Relay(RelayDebugCommands::Disable { relay }) => {
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.disable_relay(relay.clone()).await?;
                println!("{relay} is now marked as inactive");
                Ok(())
            }
            DebugCommands::Relay(RelayDebugCommands::Enable { relay }) => {
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.enable_relay(relay.clone()).await?;
                println!("{relay} is now marked as active");
                Ok(())
            }
            DebugCommands::Rollout(rollout_cmd) => rollout_cmd.handle().await,
        }
    }
}

impl RolloutDebugCommands {
    pub async fn handle(self) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        match self {
            RolloutDebugCommands::Get => {
                let Ok(threshold) = rpc.get_rollout_threshold().await else {
                    bail!("Failed to get rollout");
                };
                println!("{threshold}");
                Ok(())
            }
            RolloutDebugCommands::Reroll => {
                let Ok(threshold) = rpc.generate_new_rollout_threshold().await else {
                    bail!("Failed to get rollout");
                };
                println!("{threshold}");
                Ok(())
            }
            RolloutDebugCommands::Seed { value: seed } => {
                if rpc.set_new_rollout_threshold_seed(seed).await.is_err() {
                    bail!("Failed to update rollout seed");
                }
                Ok(())
            }
        }
    }
}
