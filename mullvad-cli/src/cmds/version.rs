use crate::{new_rpc_client, Command, Error, Result};

pub struct Version;

#[mullvad_management_interface::async_trait]
impl Command for Version {
    fn name(&self) -> &'static str {
        "version"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Shows current version, and the currently supported versions")
    }

    async fn run(&self, _: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let current_version = rpc
            .get_current_version(())
            .await
            .map_err(|error| Error::RpcFailed("Failed to obtain current version", error))?
            .into_inner();
        println!("Current version: {}", current_version);
        let version_info = rpc
            .get_version_info(())
            .await
            .map_err(|error| Error::RpcFailed("Failed to obtain version info", error))?
            .into_inner();
        println!("\tIs supported: {}", version_info.supported);

        if !version_info.suggested_upgrade.is_empty() {
            println!("\tSuggested update: {}", version_info.suggested_upgrade);
        } else {
            println!("\tNo newer version is available");
        }

        if !version_info.latest_stable.is_empty() {
            println!("\tLatest stable version: {}", version_info.latest_stable);
        }

        let settings = rpc.get_settings(()).await?.into_inner();
        if settings.show_beta_releases {
            println!("\tLatest beta version: {}", version_info.latest_beta);
        };

        Ok(())
    }
}
