#[cfg(any(target_os = "macos", target_os = "windows"))]
use anyhow::Result;
use std::path::PathBuf;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use mullvad_management_interface::MullvadProxyClient;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::super::BooleanOption;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum SplitTunnel {
    /// Display the split tunnel status and apps
    #[cfg(target_os = "macos")]
    Get,
    #[cfg(target_os = "windows")]
    Get {
        /// List processes that are currently being excluded, as well as whether they are
        /// excluded because of their executable paths or because they're subprocesses of
        /// such processes
        #[arg(long)]
        list_processes: bool,
    },
    #[cfg(target_os = "linux")]
    /// list all processes that are excluded from the tunnel
    List,
    #[cfg(target_os = "linux")]
    /// Add a PID to exclude from the tunnel
    Add {
        #[arg(required(true), num_args = 1..)]
        pids: Vec<i32>,
    },
    #[cfg(target_os = "linux")]
    /// Stop excluding a PID from the tunnel
    Delete {
        #[arg(required(true), num_args = 1..)]
        pids: Vec<i32>,
    },
    #[cfg(target_os = "linux")]
    /// Stop excluding all processes from the tunnel
    Clear,

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    /// Enable or disable split tunnel
    Set { policy: BooleanOption },

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    /// Manage applications to exclude from the tunnel
    #[clap(subcommand)]
    App(App),
}

#[derive(Subcommand, Debug)]
pub enum App {
    Add {
        #[arg(required(true), num_args = 1..)]
        paths: Vec<PathBuf>,
    },
    Remove {
        #[arg(required(true), num_args = 1..)]
        paths: Vec<PathBuf>,
    },
    Clear,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl SplitTunnel {
    pub(crate) async fn app(subcmd: App) -> Result<()> {
        let mut proxy = MullvadProxyClient::new().await?;
        match subcmd {
            App::Add { paths } => {
                for path in paths {
                    proxy.add_split_tunnel_app(&path).await?;
                    println!("Added {path:?} to excluded apps list");
                }
                Ok(())
            }
            App::Remove { paths } => {
                for path in paths {
                    proxy.remove_split_tunnel_app(&path).await?;
                    println!("Stopped excluding {path:?} from tunnel");
                }
                Ok(())
            }
            App::Clear => {
                proxy.clear_split_tunnel_apps().await?;
                println!("Stopped excluding all apps");
                Ok(())
            }
        }
    }
}
