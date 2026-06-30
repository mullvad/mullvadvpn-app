use anyhow::Result;
use clap::Subcommand;
use futures::StreamExt;
use mullvad_management_interface::MullvadProxyClient;

#[derive(Subcommand, Debug)]
pub enum Log {
    /// Set the log level of the daemon.
    SetLevel { level: Level },
    /// Set the log level of the daemon, using the `RUST_LOG` format.
    ///
    /// Set a custom log level per crate or module using the same format as the `RUST_LOG` environment variable.
    /// See the `env_logger` crate for more information: <https://docs.rs/env_logger/latest/env_logger/>
    #[expect(clippy::enum_variant_names)]
    SetRustLog { filter: String },
    /// Follow live updates to the daemon log file. Analogue to running `tail -f` on the daemon log file.
    Listen,
}

/// See <https://docs.rs/log/latest/log/enum.Level.html>
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, clap::ValueEnum)]
#[repr(usize)]
pub enum Level {
    /// Turn off logging.
    Off = 0,
    /// Log very serious errors.
    Error = 1,
    /// Log hazardous situations.
    Warn = 2,
    /// Log useful information.
    Info = 3,
    /// Log lower priority information.
    Debug = 4,
    /// Log very low priority, often extremely verbose, information.
    Trace = 5,
}

impl Log {
    pub async fn handle(self) -> Result<()> {
        match self {
            Log::SetLevel { level } => set_level(level).await,
            Log::SetRustLog { filter } => set_rust_log(filter).await,
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

async fn set_level(level: Level) -> std::result::Result<(), anyhow::Error> {
    let value = match level {
        Level::Off => mullvad_management_interface::types::LogLevelValue::Off,
        Level::Error => mullvad_management_interface::types::LogLevelValue::Error,
        Level::Warn => mullvad_management_interface::types::LogLevelValue::Warn,
        Level::Info => mullvad_management_interface::types::LogLevelValue::Info,
        Level::Debug => mullvad_management_interface::types::LogLevelValue::Debug,
        Level::Trace => mullvad_management_interface::types::LogLevelValue::Trace,
    };
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.set_log_level(mullvad_management_interface::types::LogLevel {
        level: value as i32,
    })
    .await?;
    Ok(())
}

async fn set_rust_log(filter: String) -> std::result::Result<(), anyhow::Error> {
    let mut rpc = MullvadProxyClient::new().await?;
    rpc.set_rust_log_env_filter(filter).await?;
    Ok(())
}
