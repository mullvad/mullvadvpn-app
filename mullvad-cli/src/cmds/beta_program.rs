use anyhow::{anyhow, Result};
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::BooleanOption;

#[derive(Subcommand, Debug)]
pub enum BetaProgram {
    /// Get beta notifications setting
    Get,
    /// Change beta notifications setting
    Set { policy: BooleanOption },
}

impl BetaProgram {
    pub async fn handle(self) -> Result<()> {
        match self {
            BetaProgram::Get => Self::get().await,
            BetaProgram::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(state: BooleanOption) -> Result<()> {
        if !*state && mullvad_version::VERSION.contains("beta") {
            return Err(anyhow!(
                "The beta program must be enabled while running a beta version",
            ));
        }

        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_show_beta_releases(*state).await?;

        println!("Beta program: {state}");
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let opt = BooleanOption::from(rpc.get_settings().await?.show_beta_releases);
        println!("Beta program: {opt}");
        Ok(())
    }
}
