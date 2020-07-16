use crate::{new_grpc_client, Command, Error, Result};
use clap::value_t_or_exit;

pub struct BlockWhenDisconnected;

#[async_trait::async_trait]
impl Command for BlockWhenDisconnected {
    fn name(&self) -> &'static str {
        "always-require-vpn"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control if the system service should block network access when disconnected from VPN")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change the always require VPN setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display the current always require VPN setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let block_when_disconnected = value_t_or_exit!(set_matches.value_of("policy"), String);
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
        let mut rpc = new_grpc_client().await?;
        rpc.set_block_when_disconnected(block_when_disconnected)
            .await
            .map_err(Error::GrpcClientError)?;
        println!("Changed always require VPN setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let block_when_disconnected = rpc.get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
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
