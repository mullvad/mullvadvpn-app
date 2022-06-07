use std::{ffi::OsStr, path::Path};

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
            .subcommand(create_pid_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("app", matches)) => Self::handle_app_subcommand(matches).await,
            Some(("pid", matches)) => Self::handle_pid_subcommand(matches).await,
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

fn create_pid_subcommand() -> clap::App<'static> {
    clap::App::new("pid")
        .about("Manages processes (PIDs) excluded from the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("list")
            .about("List processes that are currently being excluded, i.e. their PIDs, as well as whether \
                    they are excluded because of their executable paths or because they're subprocesses of \
                    such processes"))
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

    async fn handle_pid_subcommand(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("list", _)) => {
                let processes = new_rpc_client()
                    .await?
                    .get_excluded_processes(())
                    .await?
                    .into_inner();

                for process in &processes.processes {
                    let subproc = if process.inherited { "subprocess" } else { "" };
                    println!(
                        "{:<7}{subproc:<12}{}",
                        process.pid,
                        Path::new(&process.image)
                            .file_name()
                            .unwrap_or(OsStr::new("unknown"))
                            .to_string_lossy()
                    );
                }

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
