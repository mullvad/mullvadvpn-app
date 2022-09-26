use crate::{new_rpc_client, Command, Result};

pub struct AutoConnect;

#[mullvad_management_interface::async_trait]
impl Command for AutoConnect {
    fn name(&self) -> &'static str {
        "auto-connect"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Control the daemon auto-connect setting")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::App::new("set")
                    .about("Change auto-connect setting")
                    .arg(
                        clap::Arg::new("policy")
                            .required(true)
                            .possible_values(["on", "off"]),
                    ),
            )
            .subcommand(clap::App::new("get").about("Display the current auto-connect setting"))
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let auto_connect = set_matches.value_of("policy").expect("missing policy");
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
        let mut rpc = new_rpc_client().await?;
        rpc.set_auto_connect(auto_connect).await?;
        println!("Changed auto-connect setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let auto_connect = rpc.get_settings(()).await?.into_inner().auto_connect;
        println!("Autoconnect: {}", if auto_connect { "on" } else { "off" });
        Ok(())
    }
}
