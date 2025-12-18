use anyhow::Result;
use clap::Subcommand;
use futures::StreamExt;
use mullvad_management_interface::MullvadProxyClient;

#[derive(Subcommand, Debug)]
pub enum Log {
    /// Set the log level for the daemon
    SetLevel {
        /// The log level to set
        level: String,
    },
    /// Follow live updates to the daemon log file. Analogue to running `tail -f` on the daemon log file.
    Listen,
}

impl Log {
    pub async fn handle(self) -> Result<()> {
        match self {
            Log::SetLevel { level } => set_level(level).await,
            Log::Listen => on_listen().await,
        }
    }
}

async fn on_listen() -> std::result::Result<(), anyhow::Error> {
    let mut rpc = MullvadProxyClient::new().await?;
    let log_stream = rpc.log_listen().await?;
    log_stream
        .for_each(|log| async {
            match log {
                Ok(log) => print!("{log}"),
                Err(e) => eprint!("{e}"),
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
