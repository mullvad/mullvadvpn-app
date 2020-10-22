use crate::{new_rpc_client, Command, Result};
use mullvad_management_interface::types;

pub struct CustomDns;

#[mullvad_management_interface::async_trait]
impl Command for CustomDns {
    fn name(&self) -> &'static str {
        "custom-dns"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Configure custom DNS servers to use when connected")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Change custom DNS setting")
                    .arg(
                        clap::Arg::with_name("servers")
                            .multiple(true)
                            .help("One or more IP addresses pointing to DNS resolvers.")
                            .required(true),
                    ),
            )
            .subcommand(clap::SubCommand::with_name("reset").about("Remove all custom DNS servers"))
            .subcommand(
                clap::SubCommand::with_name("get").about("Display the current custom DNS setting"),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            self.set(set_matches.values_of_lossy("servers")).await
        } else if let Some(_matches) = matches.subcommand_matches("reset") {
            self.reset().await
        } else if let Some(_matches) = matches.subcommand_matches("get") {
            self.get().await
        } else {
            unreachable!("No custom-dns command given");
        }
    }
}

impl CustomDns {
    async fn set(&self, servers: Option<Vec<String>>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_custom_dns(types::CustomDns {
            addresses: servers.unwrap_or_default(),
        })
        .await?;
        println!("Updated custom DNS settings");
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_custom_dns(types::CustomDns { addresses: vec![] })
            .await?;
        println!("Cleared list of custom DNS servers");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let custom_dns = rpc
            .get_settings(())
            .await?
            .into_inner()
            .tunnel_options
            .unwrap()
            .generic
            .unwrap()
            .custom_dns;
        match custom_dns {
            None => println!("No DNS servers are configured"),
            Some(types::CustomDns { addresses }) => {
                println!("Custom DNS servers:");
                for server in &addresses {
                    println!("\t{}", server);
                }
            }
        }
        Ok(())
    }
}
