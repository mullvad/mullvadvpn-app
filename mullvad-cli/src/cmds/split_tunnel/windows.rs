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

    /// Manage IP/subnet exclusions from the VPN firewall
    #[clap(subcommand)]
    Ip(Ip),
}

#[derive(Subcommand, Debug)]
pub enum App {
    Add { path: PathBuf },
    Remove { path: PathBuf },
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum Ip {
    /// Add an IP network (CIDR notation, e.g. 100.64.0.0/10) to bypass the VPN firewall
    Add {
        /// IP network in CIDR notation (e.g. 100.64.0.0/10 or 192.168.1.0/24)
        network: String,
    },
    /// Remove an IP network from the exclusion list
    Remove {
        /// IP network in CIDR notation (must match exactly what was added)
        network: String,
    },
    /// List all excluded IP networks
    List,
    /// Remove all excluded IP networks
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
                    println!("  {}", path.display());
                }

                println!("Excluded IP networks:");
                for network in &settings.ip_exclusions {
                    println!("  {network}");
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
            SplitTunnel::Ip(subcmd) => Self::ip(subcmd).await,
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

    async fn ip(subcmd: Ip) -> Result<()> {
        match subcmd {
            Ip::Add { network } => {
                // Validate CIDR notation before sending
                network
                    .parse::<ipnetwork::IpNetwork>()
                    .map_err(|e| anyhow::anyhow!("Invalid CIDR notation '{network}': {e}"))?;
                MullvadProxyClient::new()
                    .await?
                    .add_split_tunnel_ip_network(network.clone())
                    .await?;
                println!("Added {network} to excluded IP networks");
                Ok(())
            }
            Ip::Remove { network } => {
                network
                    .parse::<ipnetwork::IpNetwork>()
                    .map_err(|e| anyhow::anyhow!("Invalid CIDR notation '{network}': {e}"))?;
                MullvadProxyClient::new()
                    .await?
                    .remove_split_tunnel_ip_network(network.clone())
                    .await?;
                println!("Removed {network} from excluded IP networks");
                Ok(())
            }
            Ip::List => {
                let mut rpc = MullvadProxyClient::new().await?;
                let settings = rpc.get_settings().await?.split_tunnel;
                if settings.ip_exclusions.is_empty() {
                    println!("No excluded IP networks");
                } else {
                    println!("Excluded IP networks:");
                    for network in &settings.ip_exclusions {
                        println!("  {network}");
                    }
                }
                Ok(())
            }
            Ip::Clear => {
                MullvadProxyClient::new()
                    .await?
                    .clear_split_tunnel_ip_networks()
                    .await?;
                println!("Cleared all excluded IP networks");
                Ok(())
            }
        }
    }
}
