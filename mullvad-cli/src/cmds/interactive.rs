use crate::{interactive, Command, Result};

pub struct Interactive;

#[mullvad_management_interface::async_trait]
impl Command for Interactive {
    fn name(&self) -> &'static str {
        "interactive"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name()).about("Run the interactive TUI")
    }

    async fn run(&self, _matches: &clap::ArgMatches) -> Result<()> {
        interactive::run().await;
        Ok(())
    }
}
