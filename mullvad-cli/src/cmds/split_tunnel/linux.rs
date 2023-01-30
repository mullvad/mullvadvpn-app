use crate::{new_rpc_client, Command, Result};

pub struct SplitTunnel;

#[mullvad_management_interface::async_trait]
impl Command for SplitTunnel {
    fn name(&self) -> &'static str {
        "split-tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about(
                "Manage split tunneling. To launch applications outside \
                    the tunnel, use the program 'mullvad-exclude' instead of this command.",
            )
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_pid_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("pid", pid_matches)) => Self::handle_pid_cmd(pid_matches).await,
            _ => unreachable!("unhandled comand"),
        }
    }
}

fn create_pid_subcommand() -> clap::App<'static> {
    clap::App::new("pid")
        .about("Manage processes to exclude from the tunnel")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::App::new("add").arg(clap::Arg::new("pid").required(true)))
        .subcommand(clap::App::new("delete").arg(clap::Arg::new("pid").required(true)))
        .subcommand(clap::App::new("clear"))
        .subcommand(clap::App::new("list"))
}

impl SplitTunnel {
    async fn handle_pid_cmd(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("add", matches)) => {
                let pid: i32 = matches.value_of_t_or_exit("pid");
                new_rpc_client()
                    .await?
                    .add_split_tunnel_process(pid)
                    .await?;
                Ok(())
            }
            Some(("delete", matches)) => {
                let pid: i32 = matches.value_of_t_or_exit("pid");
                new_rpc_client()
                    .await?
                    .remove_split_tunnel_process(pid)
                    .await?;
                Ok(())
            }
            Some(("clear", _)) => {
                new_rpc_client()
                    .await?
                    .clear_split_tunnel_processes(())
                    .await?;
                Ok(())
            }
            Some(("list", _)) => {
                let mut pids_stream = new_rpc_client()
                    .await?
                    .get_split_tunnel_processes(())
                    .await?
                    .into_inner();
                println!("Excluded PIDs:");

                while let Some(pid) = pids_stream.message().await? {
                    println!("    {pid}");
                }

                Ok(())
            }
            _ => unreachable!("unhandled command"),
        }
    }
}
