#[cfg(any(target_os = "macos", target_os = "windows"))]
use anyhow::Result;
#[cfg(any(target_os = "macos", target_os = "windows"))]
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
    /// list all processes that are excluded from the tunnel
    #[cfg(target_os = "linux")]
    List,
    /// Add a PID to exclude from the tunnel
    #[cfg(target_os = "linux")]
    Add {
        #[arg(required(true), num_args = 1..)]
        pids: Vec<i32>,
    },
    /// Stop excluding a PID from the tunnel
    #[cfg(target_os = "linux")]
    Delete {
        #[arg(required(true), num_args = 1..)]
        pids: Vec<i32>,
    },
    /// Stop excluding all processes from the tunnel
    #[cfg(target_os = "linux")]
    Clear,

    /// Enable or disable split tunnel
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    Set { policy: BooleanOption },

    /// Manage applications to exclude from the tunnel
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[clap(subcommand)]
    App(App),
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
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
