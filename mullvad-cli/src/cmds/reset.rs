use crate::{new_rpc_client, Command, Error, Result};
use std::io::stdin;

pub struct Reset;
#[mullvad_management_interface::async_trait]
impl Command for Reset {
    fn name(&self) -> &'static str {
        "factory-reset"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name()).about("Reset settings, caches and logs")
    }

    async fn run(&self, _: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        if Self::receive_confirmation() {
            rpc.factory_reset(())
                .await
                .map_err(|error| Error::RpcFailedExt("FAILED TO PERFORM FACTORY RESET", error))?;
            #[cfg(target_os = "linux")]
            println!("If you're running systemd, to remove all logs, you must use journalctl");
        }
        Ok(())
    }
}

impl Reset {
    fn receive_confirmation() -> bool {
        println!("Are you sure you want to disconnect, log out, delete all settings, logs and cache files for the Mullvad VPN system service? [Yes/No (default)]");
        loop {
            let mut buf = String::new();
            if let Err(e) = stdin().read_line(&mut buf) {
                eprintln!("Couldn't read from STDIN: {e}");
                return false;
            }
            match buf.trim() {
                "Yes" => return true,
                "No" | "no" | "" => return false,
                _ => println!("Unexpected response. Please enter \"Yes\" or \"No\""),
            }
        }
    }
}
