use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum AutoConnect {
    /// Display the current auto-connect setting
    Get,
    /// Change auto-connect setting
    Set { policy: BooleanOption },
}

impl AutoConnect {
    pub async fn handle(self) -> Result<()> {
        match self {
            AutoConnect::Get => Self::get().await,
            AutoConnect::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(policy: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_auto_connect(*policy).await?;
        println!("Changed auto-connect setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let auto_connect = BooleanOption::from(rpc.get_settings().await?.auto_connect);
        println!("Autoconnect: {auto_connect}");
        Ok(())
    }
}
