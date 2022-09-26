use crate::{new_rpc_client, Command, Result};

use mullvad_management_interface::{types as grpc_types, ManagementServiceClient};

use mullvad_types::relay_constraints::{ObfuscationSettings, SelectedObfuscation};

use std::convert::TryFrom;

pub struct Obfuscation;

#[mullvad_management_interface::async_trait]
impl Command for Obfuscation {
    fn name(&self) -> &'static str {
        "obfuscation"
    }

    fn clap_subcommand(&self) -> clap::App<'static> {
        clap::App::new(self.name())
            .about(
                "Manage use of obfuscation protocols for WireGuard. \
                Can make WireGuard traffic look like something else on the network. \
                Helps circumvent censorship and to establish a tunnel when on restricted networks",
            )
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(create_obfuscation_set_subcommand())
            .subcommand(create_obfuscation_get_subcommand())
    }

    async fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("set", set_matches)) => Self::handle_set(set_matches).await,
            Some(("get", _get_matches)) => Self::handle_get().await,
            _ => unreachable!("unhandled command"),
        }
    }
}

impl Obfuscation {
    async fn handle_set(matches: &clap::ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("mode", mode_matches)) => {
                let obfuscator_type = mode_matches.value_of("mode").unwrap();
                let mut rpc = new_rpc_client().await?;
                let mut settings = Self::get_obfuscation_settings(&mut rpc).await?;
                settings.selected_obfuscation = match obfuscator_type {
                    "auto" => SelectedObfuscation::Auto,
                    "off" => SelectedObfuscation::Off,
                    "udp2tcp" => SelectedObfuscation::Udp2Tcp,
                    _ => unreachable!("Unhandled obfuscator mode"),
                };
                Self::set_obfuscation_settings(&mut rpc, &settings).await?;
            }
            Some(("udp2tcp", settings_matches)) => {
                let port: String = settings_matches.value_of_t_or_exit("port");
                let mut rpc = new_rpc_client().await?;
                let mut settings = Self::get_obfuscation_settings(&mut rpc).await?;
                settings.udp2tcp.port = if port == "any" {
                    mullvad_types::relay_constraints::Constraint::Any
                } else {
                    mullvad_types::relay_constraints::Constraint::Only(
                        port.parse::<u16>().expect("Invalid port number"),
                    )
                };
                Self::set_obfuscation_settings(&mut rpc, &settings).await?;
            }
            _ => unreachable!("unhandled command"),
        }
        Ok(())
    }

    async fn handle_get() -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let obfuscation_settings = Self::get_obfuscation_settings(&mut rpc).await?;
        println!(
            "Obfuscation mode: {}",
            obfuscation_settings.selected_obfuscation
        );
        println!("udp2tcp settings: {}", obfuscation_settings.udp2tcp);
        Ok(())
    }

    async fn get_obfuscation_settings(
        rpc: &mut ManagementServiceClient,
    ) -> Result<ObfuscationSettings> {
        let settings = rpc.get_settings(()).await?.into_inner();

        let obfuscation_settings = ObfuscationSettings::try_from(
            settings
                .obfuscation_settings
                .expect("No obfuscation settings"),
        )
        .expect("failed to parse obfuscation settings");
        Ok(obfuscation_settings)
    }

    async fn set_obfuscation_settings(
        rpc: &mut ManagementServiceClient,
        settings: &ObfuscationSettings,
    ) -> Result<()> {
        let grpc_settings: grpc_types::ObfuscationSettings = settings.into();
        let _ = rpc.set_obfuscation_settings(grpc_settings).await?;
        Ok(())
    }
}

fn create_obfuscation_set_subcommand() -> clap::App<'static> {
    clap::App::new("set")
        .about("Set obfuscation settings")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::App::new("mode").about("Set obfuscation mode").arg(
                clap::Arg::new("mode")
                    .help(
                        "Specifies if obfuscation should be used with WireGuard connections. \
                        And if so, what obfuscation protocol it should use.",
                    )
                    .required(true)
                    .index(1)
                    .possible_values(["auto", "off", "udp2tcp"]),
            ),
        )
        .subcommand(
            clap::App::new("udp2tcp")
                .about("Specifies the config for the udp2tcp obfuscator")
                .setting(clap::AppSettings::ArgRequiredElseHelp)
                .arg(
                    clap::Arg::new("port")
                        .help("TCP port of remote endpoint. Either 'any' or a specific port")
                        .long("port")
                        .takes_value(true),
                ),
        )
}

fn create_obfuscation_get_subcommand() -> clap::App<'static> {
    clap::App::new("get").about("Get current obfuscation settings")
}
