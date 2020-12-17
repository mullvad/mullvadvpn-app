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
            .about(
                "Manage split tunneling. To launch applications outside \
                    the tunnel, use the program 'mullvad-exclude'.",
            )
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_pid_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("pid", Some(pid_matches)) => Self::handle_pid_cmd(pid_matches).await,
            _ => unreachable!("unhandled comand"),
        }
    }
}

fn create_pid_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("pid")
        .about("Manage processes to exclude from the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::SubCommand::with_name("add").arg(clap::Arg::with_name("pid").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("delete").arg(clap::Arg::with_name("pid").required(true)),
        )
        .subcommand(clap::SubCommand::with_name("clear"))
        .subcommand(clap::SubCommand::with_name("list"))
}

impl SplitTunnel {
    async fn handle_pid_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("add", Some(matches)) => {
                let pid = value_t_or_exit!(matches.value_of("pid"), i32);
                new_rpc_client()
                    .await?
                    .add_split_tunnel_process(pid)
                    .await?;
                Ok(())
            }
            ("delete", Some(matches)) => {
                let pid = value_t_or_exit!(matches.value_of("pid"), i32);
                new_rpc_client()
                    .await?
                    .remove_split_tunnel_process(pid)
                    .await?;
                Ok(())
            }
            ("clear", Some(_)) => {
                new_rpc_client()
                    .await?
                    .clear_split_tunnel_processes(())
                    .await?;
                Ok(())
            }
            ("list", Some(_)) => {
                let mut pids_stream = new_rpc_client()
                    .await?
                    .get_split_tunnel_processes(())
                    .await?
                    .into_inner();
                println!("Excluded PIDs:");

                while let Some(pid) = pids_stream.message().await? {
                    println!("    {}", pid);
                }

                Ok(())
            }
            _ => unreachable!("unhandled command"),
        }
    }
}
