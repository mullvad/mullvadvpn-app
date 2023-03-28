use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::on_off_parser;
use crate::Result;

#[derive(Subcommand, Debug)]
pub enum LockdownMode {
    /// Display the current lockdown mode setting
    Get,
    /// Change the lockdown mode setting
    Set {
        #[arg(value_parser = on_off_parser())]
        policy: bool,
    },
}

impl LockdownMode {
    pub async fn handle(self) -> Result<()> {
        match self {
            LockdownMode::Get => Self::get().await,
            LockdownMode::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(policy: bool) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_block_when_disconnected(policy).await?;
        println!("Changed lockdown mode setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let block_when_disconnected = rpc.get_settings().await?.block_when_disconnected;
        println!(
            "Network traffic will be {} when the VPN is disconnected",
            if block_when_disconnected {
                "blocked"
            } else {
                "allowed"
            }
        );
        Ok(())
    }
}
