use crate::{format, format::print_keygen_event, new_rpc_client, Command, Error, Result};
use mullvad_management_interface::{
    types::daemon_event::Event as EventType, ManagementServiceClient,
};

pub struct Status;

#[mullvad_management_interface::async_trait]
impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("View the state of the VPN tunnel")
            .arg(
                clap::Arg::with_name("location")
                    .long("location")
                    .short("l")
                    .help("Prints the current location and IP. Based on GeoIP lookups"),
            )
            .subcommand(
                clap::SubCommand::with_name("listen")
                    .about("Listen for VPN tunnel state changes")
                    .arg(
                        clap::Arg::with_name("verbose")
                            .short("v")
                            .help("Enables verbose output"),
                    ),
            )
    }

    async fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut rpc = new_rpc_client().await?;
        let state = rpc.get_tunnel_state(()).await?.into_inner();

        format::print_state(&state);
        if matches.is_present("location") {
            print_location(&mut rpc).await?;
        }

        if let Some(listen_matches) = matches.subcommand_matches("listen") {
            let verbose = listen_matches.is_present("verbose");

            let mut events = rpc.events_listen(()).await?.into_inner();

            while let Some(event) = events.message().await? {
                match event.event.unwrap() {
                    EventType::TunnelState(new_state) => {
                        format::print_state(&new_state);
                        use mullvad_management_interface::types::tunnel_state::State::*;
                        match new_state.state.unwrap() {
                            Connected(..) | Disconnected(..) => {
                                if matches.is_present("location") {
                                    print_location(&mut rpc).await?;
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::Settings(settings) => {
                        if verbose {
                            println!("New settings: {:#?}", settings);
                        }
                    }
                    EventType::RelayList(relay_list) => {
                        if verbose {
                            println!("New relay list: {:#?}", relay_list);
                        }
                    }
                    EventType::VersionInfo(app_version_info) => {
                        if verbose {
                            println!("New app version info: {:#?}", app_version_info);
                        }
                    }
                    EventType::KeyEvent(key_event) => {
                        if verbose {
                            print!("Key event: ");
                            print_keygen_event(&key_event);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

async fn print_location(rpc: &mut ManagementServiceClient) -> Result<()> {
    let location = rpc.get_current_location(()).await;
    let location = match location {
        Ok(response) => response.into_inner(),
        Err(status) => {
            if status.code() == mullvad_management_interface::Code::NotFound {
                println!("Location data unavailable");
                return Ok(());
            } else {
                return Err(Error::GrpcClientError(status));
            }
        }
    };
    if !location.hostname.is_empty() {
        println!("Relay: {}", location.hostname);
    }
    if !location.ipv4.is_empty() {
        println!("IPv4: {}", location.ipv4);
    }
    if !location.ipv6.is_empty() {
        println!("IPv6: {}", location.ipv6);
    }

    print!("Location: ");
    if !location.city.is_empty() {
        print!("{}, ", location.city);
    }
    println!("{}", location.country);

    println!(
        "Position: {:.5}°N, {:.5}°W",
        location.latitude, location.longitude
    );
    Ok(())
}
