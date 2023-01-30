use crate::{new_rpc_client, Command, Error, Result};

pub struct BetaProgram;

#[mullvad_management_interface::async_trait]
impl Command for BetaProgram {
    fn name(&self) -> &'static str {
        "beta-program"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Receive notifications about beta updates")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::App::new("set")
                    .about("Change beta notifications setting")
                    .arg(
                        clap::Arg::new("policy")
                            .required(true)
                            .possible_values(["on", "off"]),
                    ),
            )
            .subcommand(clap::App::new("get").about("Get beta notifications setting"))
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("get", _)) => {
                let mut rpc = new_rpc_client().await?;
                let settings = rpc.get_settings(()).await?.into_inner();
                let enabled_str = if settings.show_beta_releases {
                    "on"
                } else {
                    "off"
                };
                println!("Beta program: {enabled_str}");
                Ok(())
            }
            Some(("set", matches)) => {
                let enable_str = matches.value_of("policy").expect("missing policy");
                let enable = enable_str == "on";

                if !enable && mullvad_version::VERSION.contains("beta") {
                    return Err(Error::InvalidCommand(
                        "The beta program must be enabled while running a beta version",
                    ));
                }

                let mut rpc = new_rpc_client().await?;
                rpc.set_show_beta_releases(enable).await?;

                println!("Beta program: {enable_str}");
                Ok(())
            }
            _ => {
                unreachable!("unhandled comand");
            }
        }
    }
}
