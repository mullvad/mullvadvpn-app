use clap;
use {Command, Result};

use mullvad_ipc_client::DaemonRpcClient;

pub struct Autoconnect;

impl Command for Autoconnect {
    fn name(&self) -> &'static str {
        "autoconnect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control the daemon autoconnect setting")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change autoconnect setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get").about("Display the current autoconnect setting"),
            )
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let autoconnect = value_t_or_exit!(set_matches.value_of("policy"), String);
            self.set(autoconnect == "on")
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get()
        } else {
            unreachable!("No autoconnect command given");
        }
    }
}

impl Autoconnect {
    fn set(&self, autoconnect: bool) -> Result<()> {
        let mut rpc = DaemonRpcClient::new()?;
        rpc.set_autoconnect(autoconnect)?;
        println!("Changed autoconnect sharing setting");
        Ok(())
    }

    fn get(&self) -> Result<()> {
        let mut rpc = DaemonRpcClient::new()?;
        let autoconnect = rpc.get_autoconnect()?;
        println!("Autoconnect: {}", if autoconnect { "on" } else { "off" });
        Ok(())
    }
}
