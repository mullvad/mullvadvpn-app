use crate::{new_rpc_client, Command, Error, Result};

pub struct Version;

#[mullvad_management_interface::async_trait]
impl Command for Version {
    fn name(&self) -> &'static str {
        "version"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about("Shows current version, and the currently supported versions")
    }

    async fn run(&self, _: &clap::ArgMatches) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let current_version = rpc
            .get_current_version(())
            .await
            .map_err(|error| Error::RpcFailedExt("Failed to obtain current version", error))?
            .into_inner();
        println!("{:21}: {}", "Current version", current_version);
        let version_info = rpc
            .get_version_info(())
            .await
            .map_err(|error| Error::RpcFailedExt("Failed to obtain version info", error))?
            .into_inner();
        println!("{:21}: {}", "Is supported", version_info.supported);

        if !version_info.suggested_upgrade.is_empty() {
            println!(
                "{:21}: {}",
                "Suggested upgrade", version_info.suggested_upgrade
            );
        } else {
            println!("{:21}: none", "Suggested upgrade");
        }

        if !version_info.latest_stable.is_empty() {
            println!(
                "{:21}: {}",
                "Latest stable version", version_info.latest_stable
            );
        }

        let settings = rpc.get_settings(()).await?.into_inner();
        if settings.show_beta_releases {
            println!("{:21}: {}", "Latest beta version", version_info.latest_beta);
        };

        Ok(())
    }
}
