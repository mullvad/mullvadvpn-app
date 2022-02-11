use crate::{new_rpc_client, Command, Result};

pub struct SplitTunnel;

#[mullvad_management_interface::async_trait]
impl Command for SplitTunnel {
    fn name(&self) -> &'static str {
        "split-tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Set options for applications to exclude from the tunnel")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_app_subcommand())
            .subcommand(
                clap::App::new("set")
                    .about("Enable or disable split tunnel")
                    .arg(
                        clap::Arg::new("policy")
                            .required(true)
                            .possible_values(&["on", "off"]),
                    ),
            )
            .subcommand(clap::App::new("get").about("Display the split tunnel status"))
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("app", matches)) => Self::handle_app_subcommand(matches).await,
            Some(("get", _)) => self.get().await,
            Some(("set", matches)) => {
                let enabled = matches.value_of("policy").expect("missing policy");
                self.set(enabled == "on").await
            }
            _ => {
                unreachable!("unhandled command");
            }
        }
    }
}

fn create_app_subcommand() -> clap::App<'static> {
    clap::App::new("app")
        .about("Manage applications to exclude from the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("list"))
        .subcommand(clap::App::new("add").arg(clap::Arg::new("path").required(true)))
        .subcommand(clap::App::new("remove").arg(clap::Arg::new("path").required(true)))
        .subcommand(clap::App::new("clear"))
}

impl SplitTunnel {
    async fn handle_app_subcommand(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("list", _)) => {
                let paths = new_rpc_client()
                    .await?
                    .get_settings(())
                    .await?
                    .into_inner()
                    .split_tunnel
                    .unwrap()
                    .apps;

                println!("Excluded applications:");
                for path in &paths {
                    println!("    {}", path);
                }

                Ok(())
            }
            Some(("add", matches)) => {
                let path: String = matches.value_of_t_or_exit("path");
                new_rpc_client().await?.add_split_tunnel_app(path).await?;
                Ok(())
            }
            Some(("remove", matches)) => {
                let path: String = matches.value_of_t_or_exit("path");
                new_rpc_client()
                    .await?
                    .remove_split_tunnel_app(path)
                    .await?;
                Ok(())
            }
            Some(("clear", _)) => {
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
