use clap;
use {Command, Result};

use rpc;

pub struct Shutdown;

impl Command for Shutdown {
    fn name(&self) -> &'static str {
        "shutdown"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name()).about("Makes the backend daemon quit")
    }

    fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        rpc::call("shutdown", &[] as &[u8; 0])
    }
}
