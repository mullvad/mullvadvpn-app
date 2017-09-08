pub struct CustomRelay;

use Command;
use Result;
use clap;
use mullvad_types::relay_endpoint::RelayEndpoint;

use rpc;

use talpid_types::net::TransportProtocol;

impl Command for CustomRelay {
    fn name(&self) -> &'static str {
        "relay"
    }

    fn clap_subcommand(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Set a custom remote relay to connect to")
            .setting(clap::AppSettings::SubcommandRequired)
            .subcommand(clap::SubCommand::with_name("set")
                .about("Set a custom remote relay")
                .arg(clap::Arg::with_name("host")
                     .help("The host name or IP of the remote relay")
                     .required(true))
                .arg(clap::Arg::with_name("port")
                     .help("The port of the remote relay")
                     .required(true))
                .arg(clap::Arg::with_name("protocol")
                     .help("The transport protocol. UDP is recommended as it usually results in higher throughput that TCP")
                     .possible_values(&["udp", "tcp"])
                     .default_value("udp")))
    }

    fn run(&self, matches: &clap::ArgMatches) -> Result<()> {
        if let Some(set_matches) = matches.subcommand_matches("set") {
            let host = value_t_or_exit!(set_matches.value_of("host"), String);
            let port = value_t_or_exit!(set_matches.value_of("port"), u16);
            let protocol = value_t_or_exit!(set_matches.value_of("protocol"), TransportProtocol);

            self.set(host, port, protocol)
        } else {
            unreachable!("No sub command given");
        }
    }
}

impl CustomRelay {
    fn set(&self, host: String, port: u16, protocol: TransportProtocol) -> Result<()> {
        let relay_endpoint = RelayEndpoint {
            host,
            port,
            protocol,
        };

        rpc::call(
            "set_custom_relay",
            &[relay_endpoint],
        )
                .map(|_: Option<()>| println!("Custom remote relay set"))
    }
}
