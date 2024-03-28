use anyhow::Result;
use std::path::PathBuf;

use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::super::BooleanOption;

/// Set options for applications to exclude from the tunnel.
#[derive(Subcommand, Debug)]
pub enum SplitTunnel {
    /// Display the split tunnel status and apps
    Get,

    /// Enable or disable split tunnel
    Set { policy: BooleanOption },

    /// Manage applications to exclude from the tunnel
    #[clap(subcommand)]
    App(App),
}

#[derive(Subcommand, Debug)]
pub enum App {
    Add { path: PathBuf },
    Remove { path: PathBuf },
    Clear,
}

impl SplitTunnel {
    pub async fn handle(self) -> Result<()> {
        match self {
            SplitTunnel::Get => {
                let mut rpc = MullvadProxyClient::new().await?;
                let settings = rpc.get_settings().await?.split_tunnel;

                let enable_exclusions = BooleanOption::from(settings.enable_exclusions);

                println!("Split tunneling state: {enable_exclusions}");

                println!("Excluded applications:");
                for path in &settings.apps {
                    println!("{}", path.display());
                }

                Ok(())
            }
            SplitTunnel::Set { policy } => {
                let mut rpc = MullvadProxyClient::new().await?;
                rpc.set_split_tunnel_state(*policy).await?;
                println!("Split tunnel policy: {policy}");
                Ok(())
            }
            SplitTunnel::App(subcmd) => Self::app(subcmd).await,
        }
    }

    async fn app(subcmd: App) -> Result<()> {
        match subcmd {
            App::Add { path } => {
                MullvadProxyClient::new()
                    .await?
                    .add_split_tunnel_app(path)
                    .await?;
                println!("Added path to excluded apps list");
                Ok(())
            }
            App::Remove { path } => {
                MullvadProxyClient::new()
                    .await?
                    .remove_split_tunnel_app(path)
                    .await?;
                println!("Stopped excluding app from tunnel");
                Ok(())
            }
            App::Clear => {
                MullvadProxyClient::new()
                    .await?
                    .clear_split_tunnel_apps()
                    .await?;
                println!("Stopped excluding all apps");
                Ok(())
            }
        }
    }
}
