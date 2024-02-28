use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::relay_constraints::{Constraint, RelayConstraints, RelaySettings};

#[derive(clap::Subcommand, Debug)]
pub enum DebugCommands {
    /// Block all internet connection by setting an invalid relay constraint.
    BlockConnection,
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
        }
    }
}
