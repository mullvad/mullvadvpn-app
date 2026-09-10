use anyhow::Result;
use std::{ffi::OsStr, path::Path};

use mullvad_management_interface::MullvadProxyClient;

use crate::cmds::split_tunnel::shared::SplitTunnel;

use super::super::BooleanOption;

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
}
