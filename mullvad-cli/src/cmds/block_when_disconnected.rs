use crate::{new_rpc_client, Command, Result};

pub struct BlockWhenDisconnected;

#[mullvad_management_interface::async_trait]
impl Command for BlockWhenDisconnected {
    fn name(&self) -> &'static str {
        "lockdown-mode"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Control if the system service should block network access when disconnected from VPN")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::App::new("set")
                    .about("Change the lockdown mode setting")
                    .arg(
                        clap::Arg::new("policy")
                            .required(true)
                            .possible_values(["on", "off"]),
                    ),
            )
            .subcommand(
                clap::App::new("get")
                    .about("Display the current lockdown mode setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let block_when_disconnected = set_matches.value_of("policy").expect("missing policy");
            self.set(block_when_disconnected == "on").await
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else {
            unreachable!("No block-when-disconnected command given");
        }
    }
}

impl BlockWhenDisconnected {
    async fn set(&self, block_when_disconnected: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_block_when_disconnected(block_when_disconnected)
            .await?;
        println!("Changed lockdown mode setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let block_when_disconnected = rpc
            .get_settings(())
            .await?
            .into_inner()
            .block_when_disconnected;
        println!(
            "Network traffic will be {} when the VPN is disconnected",
            if block_when_disconnected {
                "blocked"
            } else {
                "allowed"
            }
        );
        Ok(())
    }
}
