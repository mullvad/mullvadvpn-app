use {Command, Result};
use clap;

use mullvad_types::version;
use rpc;

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
        let current_version: String = rpc::call("get_current_version", &[] as &[u8; 0])?;
        println!("Current version: {}", current_version);
        let version_info: version::AppVersionInfo = rpc::call("get_version_info", &[] as &[u8; 0])?;
        println!("Supported: {}", version_info.current_is_supported);
        println!("Latest releases:");
        println!("\tlatest stable: {}", version_info.latest.latest_stable);
        println!("\tlatest: {}", version_info.latest.latest);
        Ok(())
    }
}
