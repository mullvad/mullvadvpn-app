use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

use super::on_off_parser;
use crate::{Error, Result};

#[derive(Subcommand, Debug)]
pub enum BetaProgram {
    /// Get beta notifications setting
    Get,
    /// Change beta notifications setting
    Set {
        #[arg(value_parser = on_off_parser())]
        policy: bool,
    },
}

impl BetaProgram {
    pub async fn handle(self) -> Result<()> {
        match self {
            BetaProgram::Get => Self::get().await,
            BetaProgram::Set { policy } => Self::set(policy).await,
        }
    }

    async fn set(enable: bool) -> Result<()> {
        if !enable && mullvad_version::VERSION.contains("beta") {
            return Err(Error::InvalidCommand(
                "The beta program must be enabled while running a beta version",
            ));
        }

        let mut rpc = MullvadProxyClient::new().await?;
        rpc.set_show_beta_releases(enable).await?;

        if enable {
            println!("Beta program: on");
        } else {
            println!("Beta program: off");
        }
        Ok(())
    }

    async fn get() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        if rpc.get_settings().await?.show_beta_releases {
            println!("Beta program: on");
        } else {
            println!("Beta program: off");
        }
        Ok(())
    }
}
