use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum LockdownMode {
    /// Display the current lockdown mode setting
    Get,
    /// Change the lockdown mode setting
    Set { policy: BooleanOption },
}

impl LockdownMode {
    pub async fn handle(self) -> Result<()> {
        match self {
            LockdownMode::Get => Self::get().await,
            LockdownMode::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(policy: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_block_when_disconnected(*policy).await?;
        println!("Changed lockdown mode setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let state = BooleanOption::from(rpc.get_settings().await?.block_when_disconnected);
        println!("Block traffic when the VPN is disconnected: {state}");
        Ok(())
    }
}
