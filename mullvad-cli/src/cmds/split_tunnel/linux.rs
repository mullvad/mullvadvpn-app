use crate::{new_rpc_client, Command, Result};
use clap::value_t_or_exit;

pub struct SplitTunnel;

#[async_trait::async_trait]
impl Command for SplitTunnel {
    fn name(&self) -> &'static str {
        "split-tunnel"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Manage split tunneling")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_pid_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("pid", Some(pid_matches)) => Self::handle_pid_cmd(pid_matches),
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
    fn handle_pid_cmd(matches: &clap::ArgMatches<'_>) -> Result<()> {
        match matches.subcommand() {
            ("add", Some(matches)) => {
                let pid = value_t_or_exit!(matches.value_of("pid"), i32);
                new_rpc_client()?.add_split_tunnel_process(pid)?;
                Ok(())
            }
            ("delete", Some(matches)) => {
                let pid = value_t_or_exit!(matches.value_of("pid"), i32);
                new_rpc_client()?.remove_split_tunnel_process(pid)?;
                Ok(())
            }
            ("clear", Some(_)) => {
                new_rpc_client()?.clear_split_tunnel_processes()?;
                Ok(())
            }
            ("list", Some(_)) => {
                let pids = new_rpc_client()?.get_split_tunnel_processes()?;
                println!("Excluded PIDs:");

                for pid in pids.iter() {
                    println!("    {}", pid);
                }

                Ok(())
            }
            _ => unreachable!("unhandled command"),
        }
    }
}
