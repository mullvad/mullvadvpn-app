use anyhow::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::super::BooleanOption;

/// Set options for applications to exclude from the tunnel.
#[derive(Subcommand, Debug)]
pub enum SplitTunnel {
    /// Display the split tunnel status and apps
    Get {
        /// List processes that are currently being excluded, as well as whether they are
        /// excluded because of their executable paths or because they're subprocesses of
        /// such processes
        #[arg(long)]
        list_processes: bool,
    },

    /// Enable or disable split tunnel
    Set { policy: BooleanOption },

    /// Manage applications to exclude from the tunnel
    #[clap(subcommand)]
    App(App),
}

#[derive(Subcommand, Debug)]
pub enum App {
    #[command(hide = true)]
    List,
    Add {
        path: PathBuf,
    },
    Remove {
        path: PathBuf,
    },
    Clear,
}

impl SplitTunnel {
    pub async fn handle(self) -> Result<()> {
        match self {
            SplitTunnel::Get { list_processes } => {
                let mut rpc = MullvadProxyClient::new().await?;
                let settings = rpc.get_settings().await?.split_tunnel;

                let enable_exclusions = BooleanOption::from(settings.enable_exclusions);

                println!("Split tunneling state: {enable_exclusions}");

                println!("Excluded applications:");
                for path in &settings.apps {
                    println!("{}", path.display());
                }

                if list_processes {
                    let processes = rpc.get_excluded_processes().await?;
                    for process in &processes {
                        let subproc = if process.inherited { "subprocess" } else { "" };
                        println!(
                            "{:<7}{subproc:<12}{}",
                            process.pid,
                            Path::new(&process.image)
                                .file_name()
                                .unwrap_or(OsStr::new("unknown"))
                                .to_string_lossy()
                        );
                    }
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
            App::List => {
                let paths = MullvadProxyClient::new()
                    .await?
                    .get_settings()
                    .await?
                    .split_tunnel
                    .apps;

                println!("Excluded applications:");
                for path in &paths {
                    println!("{}", path.display());
                }

                Ok(())
            }
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
