use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct NetworkCheck;

const SUBCOMMAND_DESCRIPTION: &'static str =
"Control the macOS network check setting. Allowing the check leaks DNS queries for `captive.apple.com`. Allowing the
connectivity check allows macOS to get online quicker after sleep and after connecting to new WiFi networks";

#[mullvad_management_interface::async_trait]
impl Command for NetworkCheck {
    fn name(&self) -> &'static str {
        "macos-network-check"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about(SUBCOMMAND_DESCRIPTION)
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Toggle macOS network check setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["allow", "block"]),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display current macOS network check setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let allow_network_check = value_t_or_exit!(set_matches.value_of("policy"), String);
            self.set(allow_network_check == "allow").await
        } else if let Some(_get_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else {
            unreachable!("No macOS network check given")
        }
    }
}

impl NetworkCheck {
    async fn set(&self, allow_network_check: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_allow_macos_network_check(allow_network_check)
            .await?;
        println!("Changed macOS network check setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let allow_network_check = rpc
            .get_settings(())
            .await?
            .into_inner()
            .allow_macos_network_check;
        println!(
            "macOS network check setting: {}",
            if allow_network_check {
                "allow"
            } else {
                "block"
            }
        );
        Ok(())
    }
}
