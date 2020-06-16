use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct Lan;

#[async_trait::async_trait]
impl Command for Lan {
    fn name(&self) -> &'static str {
        "lan"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control the allow local network sharing setting")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change allow LAN setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["allow", "block"]),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display the current local network sharing setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let allow_lan = value_t_or_exit!(set_matches.value_of("policy"), String);
            self.set(allow_lan == "allow")
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get()
        } else {
            unreachable!("No lan command given");
        }
    }
}

impl Lan {
    fn set(&self, allow_lan: bool) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.set_allow_lan(allow_lan)?;
        println!("Changed local network sharing setting");
        Ok(())
    }

    fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let allow_lan = rpc.get_settings()?.allow_lan;
        println!(
            "Local network sharing setting: {}",
            if allow_lan { "allow" } else { "block" }
        );
        Ok(())
    }
}
