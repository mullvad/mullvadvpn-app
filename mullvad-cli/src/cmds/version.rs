use crate::{new_rpc_client, Command, Result};

pub struct Version;

#[async_trait::async_trait]
impl Command for Version {
    fn name(&self) -> &'static str {
        "version"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Shows current version, and the currently supported versions")
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client()?;
        let current_version = rpc.get_current_version()?;
        println!("Current version: {}", current_version);
        let version_info = rpc.get_version_info()?;
        println!("\tIs supported: {}", version_info.supported);

        match version_info.suggested_upgrade {
            Some(version) => println!("\tSuggested update: {}", version),
            None => println!("\tNo newer version is available"),
        }

        if !version_info.latest_stable.is_empty() {
            println!("\tLatest stable version: {}", version_info.latest_stable);
        }

        let settings = rpc.get_settings()?;
        if settings.show_beta_releases {
            println!("\t Latest beta version: {}", version_info.latest_beta);
        };

        Ok(())
    }
}
