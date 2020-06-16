use crate::{new_rpc_client, Command, Result};
use std::io::stdin;

pub struct Reset;
#[async_trait::async_trait]
impl Command for Reset {
    fn name(&self) -> &'static str {
        "factory-reset"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("Reset settings, caches and logs")
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        if Self::receive_confirmation() {
            if rpc.factory_reset().is_err() {
                eprintln!("FAILED TO PERFORM FACTORY RESET");
            } else {
                #[cfg(target_os = "linux")]
                println!("If you're running systemd, to remove all logs, you must use journalctl");
            }
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
                eprintln!("Couldn't read from STDIN - {}", e);
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
