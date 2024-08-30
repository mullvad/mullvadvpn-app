use anyhow::Result;
use clap::Subcommand;
use mullvad_management_interface::MullvadProxyClient;

#[derive(Subcommand, Debug)]
pub enum Logging {
    /// Set the log level for the daemon
    SetLevel {
        /// The log level to set
        level: String,
    },
}

impl Logging {
    pub async fn handle(self) -> Result<()> {
        match self {
            Logging::SetLevel { level } => set_level(level).await,
        }
    }
}

async fn set_level(level: String) -> std::result::Result<(), anyhow::Error> {
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.set_log_filter(level).await?;
    Ok(())
}
