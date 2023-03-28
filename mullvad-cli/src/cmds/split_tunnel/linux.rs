use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use crate::Result;

/// Manage split tunneling. To launch applications outside the tunnel, use the program
/// 'mullvad-exclude' instead of this command
#[derive(Subcommand, Debug)]
pub enum SplitTunnel {
    /// List all processes that are excluded from the tunnel
    List,
    /// Add a PID to exclude from the tunnel
    Add { pid: i32 },
    /// Stop excluding a PID from the tunnel
    Delete { pid: i32 },
    /// Stop excluding all processes from the tunnel
    Clear,
}

impl SplitTunnel {
    pub async fn handle(self) -> Result<()> {
        match self {
            SplitTunnel::List => {
                let pids = MullvadProxyClient::new()
                    .await?
                    .get_split_tunnel_processes()
                    .await?;

                println!("Excluded PIDs:");
                for pid in &pids {
                    println!("{pid}");
                }

                Ok(())
            }
            SplitTunnel::Add { pid } => {
                MullvadProxyClient::new()
                    .await?
                    .add_split_tunnel_process(pid)
                    .await?;
                println!("Excluding process");
                Ok(())
            }
            SplitTunnel::Delete { pid } => {
                MullvadProxyClient::new()
                    .await?
                    .remove_split_tunnel_process(pid)
                    .await?;
                println!("Stopped excluding process");
                Ok(())
            }
            SplitTunnel::Clear => {
                MullvadProxyClient::new()
                    .await?
                    .clear_split_tunnel_processes()
                    .await?;
                println!("Stopped excluding all processes");
                Ok(())
            }
        }
    }
}
