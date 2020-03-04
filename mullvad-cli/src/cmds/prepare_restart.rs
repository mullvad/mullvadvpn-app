use crate::{new_rpc_client, Command, Result};

pub struct PrepareRestart;

impl Command for PrepareRestart {
    fn name(&self) -> &'static str {
        "prepare-restart"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).setting(clap::AppSettings::Hidden)
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        rpc.prepare_restart()?;
        Ok(())
    }
}
