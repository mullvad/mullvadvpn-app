use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct SplitTunnel;

#[mullvad_management_interface::async_trait]
impl Command for SplitTunnel {
    fn name(&self) -> &'static str {
        "split-tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Set options for applications to exclude from the tunnel")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_app_subcommand())
            .subcommand(
                clap::SubCommand::with_name("set")
                    .about("Enable or disable split tunnel")
                    .arg(
                        clap::Arg::with_name("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(clap::SubCommand::with_name("get").about("Display the split tunnel status"))
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("app", Some(matches)) => Self::handle_app_subcommand(matches).await,
            ("get", _) => self.get().await,
            ("set", Some(matches)) => {
                let enabled = value_t_or_exit!(matches.value_of("policy"), String);
                self.set(enabled == "on").await
            }
            _ => {
                unreachable!("unhandled command");
            }
        }
    }
}

fn create_app_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("app")
        .about("Manage applications to exclude from the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("list"))
        .subcommand(
            clap::SubCommand::with_name("add").arg(clap::Arg::with_name("path").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("remove").arg(clap::Arg::with_name("path").required(true)),
        )
        .subcommand(clap::SubCommand::with_name("clear"))
}

impl SplitTunnel {
    async fn handle_app_subcommand(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("list", Some(_)) => {
                let mut paths = new_rpc_client()
                    .await?
                    .get_split_tunnel_apps(())
                    .await?
                    .into_inner();

                println!("Excluded applications:");
                while let Some(path) = paths.message().await? {
                    println!("    {}", path);
                }

                Ok(())
            }
            ("add", Some(matches)) => {
                let path = value_t_or_exit!(matches.value_of("path"), String);
                new_rpc_client().await?.add_split_tunnel_app(path).await?;
                Ok(())
            }
            ("remove", Some(matches)) => {
                let path = value_t_or_exit!(matches.value_of("path"), String);
                new_rpc_client()
                    .await?
                    .remove_split_tunnel_app(path)
                    .await?;
                Ok(())
            }
            ("clear", Some(_)) => {
                new_rpc_client().await?.clear_split_tunnel_apps(()).await?;
                Ok(())
            }
            _ => unreachable!("unhandled subcommand"),
        }
    }

    async fn set(&self, enabled: bool) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        rpc.set_split_tunnel_state(enabled).await?;
        println!("Changed split tunnel setting");
        Ok(())
    }

    async fn get(&self) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let enabled = rpc
            .get_settings(())
            .await?
            .into_inner()
            .split_tunnel
            .unwrap()
            .enable_exclusions;
        println!(
            "Split tunnel status: {}",
            if enabled { "on" } else { "off" }
        );
        Ok(())
    }
}
