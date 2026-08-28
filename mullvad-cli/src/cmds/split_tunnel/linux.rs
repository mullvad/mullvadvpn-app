use anyhow::Result;
use mullvad_management_interface::MullvadProxyClient;

use crate::cmds::split_tunnel::shared::SplitTunnel;

/// Manage split tunneling. To launch applications outside the tunnel, use the program
/// 'mullvad-exclude' instead of this command
impl SplitTunnel {
    pub async fn handle(self) -> Result<()> {
        let mut proxy = MullvadProxyClient::new().await?;
        match self {
            SplitTunnel::List => {
                let pids = proxy.get_split_tunnel_processes().await?;

                println!("Excluded PIDs:");
                for pid in &pids {
                    println!("{pid}");
                }

                Ok(())
            }
            SplitTunnel::Add { pids } => {
                for pid in pids {
                    proxy.add_split_tunnel_process(pid).await?;
                    println!("Excluding process {pid:?} ");
                }
                Ok(())
            }
            SplitTunnel::Delete { pids } => {
                for pid in pids {
                    proxy.remove_split_tunnel_process(pid).await?;
                    println!("Stopped excluding process {pid:?}");
                }
                Ok(())
            }
            SplitTunnel::Clear => {
                proxy.clear_split_tunnel_processes().await?;
                println!("Stopped excluding all processes");
                Ok(())
            }
        }
    }
}
