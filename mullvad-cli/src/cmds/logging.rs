use anyhow::Result;
use clap::Subcommand;
use futures::StreamExt;
use mullvad_management_interface::MullvadProxyClient;

#[derive(Subcommand, Debug)]
pub enum Logging {
    /// Set the log level for the daemon
    SetLevel {
        /// The log level to set
        level: String,
    },

    Listen,
}

impl Logging {
    pub async fn handle(self) -> Result<()> {
        match self {
            Logging::SetLevel { level } => set_level(level).await,
            Logging::Listen => on_listen().await,
        }
    }
}

async fn on_listen() -> std::result::Result<(), anyhow::Error> {
    let mut rpc = MullvadProxyClient::new().await?;
    let log_stream = rpc.log_listen().await?;
    log_stream
        .for_each(|log| async {
            match log {
                Ok(log) => print!("{}", log), // newlines?
                Err(e) => eprint!("Error: {}", e),
            }
        })
        .await;
    Ok(())
}

async fn set_level(level: String) -> std::result::Result<(), anyhow::Error> {
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.set_log_filter(level).await?;
    Ok(())
}
