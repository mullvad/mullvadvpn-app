use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum Lan {
    /// Display the current local network sharing setting
    Get,

    /// Change allow LAN setting
    Set {
        #[arg(value_parser = BooleanOption::custom_parser("allow", "block"))]
        policy: BooleanOption,
    },
}

impl Lan {
    pub async fn handle(self) -> Result<()> {
        match self {
            Lan::Get => Self::get().await,
            Lan::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(policy: BooleanOption) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_allow_lan(*policy).await?;
        println!("Changed local network sharing setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let allow_lan =
            BooleanOption::with_labels(rpc.get_settings().await?.allow_lan, "allow", "block");
        println!("Local network sharing setting: {allow_lan}");
        Ok(())
    }
}
