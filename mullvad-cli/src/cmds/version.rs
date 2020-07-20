use crate::{new_grpc_client, Command, Error, Result};

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

    async fn run(&self, _: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_grpc_client().await?;
        let current_version = rpc.get_current_version(()).await?.into_inner();
        println!("Current version: {}", current_version);
        let version_info = rpc.get_version_info(()).await?.into_inner();
        println!("\tIs supported: {}", version_info.supported);

        if !version_info.suggested_upgrade.is_empty() {
            println!("\tSuggested update: {}", version_info.suggested_upgrade);
        } else {
            println!("\tNo newer version is available");
        }

        if !version_info.latest_stable.is_empty() {
            println!("\tLatest stable version: {}", version_info.latest_stable);
        }

        let settings = rpc
            .get_settings(())
            .await
            .map_err(Error::GrpcClientError)?
            .into_inner();
        if settings.show_beta_releases {
            println!("\t Latest beta version: {}", version_info.latest_beta);
        };

        Ok(())
    }
}
