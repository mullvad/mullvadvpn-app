use crate::{Error, new_grpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct AutoConnect;

#[async_trait::async_trait]
impl Command for AutoConnect {
    fn name(&self) -> &'static str {
        "auto-connect"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Control the daemon auto-connect setting")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change auto-connect setting")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("get")
                    .about("Display the current auto-connect setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let auto_connect = value_t_or_exit!(set_matches.value_of("policy"), String);
            self.set(auto_connect == "on").await
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else {
            unreachable!("No auto-connect command given");
        }
    }
}

impl AutoConnect {
    async fn set(&self, auto_connect: bool) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        rpc.set_auto_connect(auto_connect)
            .await
            .map_err(Error::GrpcClientError)?;
        println!("Changed auto-connect sharing setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let auto_connect = rpc.get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner()
            .auto_connect;
        println!("Autoconnect: {}", if auto_connect { "on" } else { "off" });
        Ok(())
    }
}
