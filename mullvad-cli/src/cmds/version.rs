use crate::{new_rpc_client, Command, Result};

pub struct Version;

impl Command for Version {
    fn name(&self) -> &'static str {
        "version"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Shows current version, and the currently supported versions")
    }

    fn run(&self, _: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let current_version = rpc.get_current_version()?;
        println!("Current version: {}", current_version);
        let version_info = rpc.get_version_info()?;
        println!("Supported: {}", version_info.current_is_supported);
        println!("Latest releases:");
        println!("\tlatest stable: {}", version_info.latest.latest_stable);
        println!("\tlatest: {}", version_info.latest.latest);
        Ok(())
    }
}
