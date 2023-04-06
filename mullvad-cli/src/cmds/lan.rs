use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::on_off_parser_custom;

#[derive(Subcommand, Debug)]
pub enum Lan {
    /// Display the current local network sharing setting
    Get,

    /// Change allow LAN setting
    Set {
        #[arg(value_parser = on_off_parser_custom("allow", "block"))]
        policy: bool,
    },
}

impl Lan {
    pub async fn handle(self) -> Result<()> {
        match self {
            Lan::Get => Self::get().await,
            Lan::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(policy: bool) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_allow_lan(policy).await?;
        println!("Changed local network sharing setting");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let allow_lan = rpc.get_settings().await?.allow_lan;
        println!(
            "Local network sharing setting: {}",
            if allow_lan { "allow" } else { "block" }
        );
        Ok(())
    }
}
