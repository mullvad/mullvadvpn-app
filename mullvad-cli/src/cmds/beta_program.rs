use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct BetaProgram;

impl Command for BetaProgram {
    fn name(&self) -> &'static str {
        "beta-program"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Receive notifications about beta updates")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change beta notifications setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(clap::SubCommand::with_name("get").about("Get beta notifications setting"))
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("get", Some(_)) => {
                let mut rpc = new_rpc_client()?;
                let settings = rpc.get_settings()?;
                let enabled_str = if settings.get_show_beta_releases().unwrap_or(false) {
                    "on"
                } else {
                    "off"
                };
                println!("Beta program: {}", enabled_str);
                Ok(())
            }
            ("set", Some(matches)) => {
                let enabled_str = value_t_or_exit!(matches.value_of("policy"), String);

                let mut rpc = new_rpc_client()?;
                rpc.set_show_beta_releases(enabled_str == "on")?;

                println!("Beta program: {}", enabled_str);
                Ok(())
            }
            _ => {
                unreachable!("unhandled comand");
            }
        }
    }
}
