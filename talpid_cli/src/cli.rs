use clap::{Arg, App, ArgMatches};
use std::path::PathBuf;

use talpid_core::net::RemoteAddr;

#[cfg(all(unix, not(target_os="macos")))]
const DEFAULT_PLUGIN_PATH: &'static str = "./target/debug/libtalpid_openvpn_plugin.so";
#[cfg(target_os="macos")]
const DEFAULT_PLUGIN_PATH: &'static str = "./target/debug/libtalpid_openvpn_plugin.dylib";
#[cfg(windows)]
const DEFAULT_PLUGIN_PATH: &'static str = "./target/debug/libtalpid_openvpn_plugin.dll";


pub struct Args {
    pub binary: String,
    pub plugin_path: PathBuf,
    pub config: PathBuf,
    pub remotes: Vec<RemoteAddr>,
    pub verbosity: u64,
}

pub fn parse_args_or_exit() -> Args {
    let matches = get_matches();
    let remotes = values_t!(matches.values_of("remotes"), RemoteAddr).unwrap_or_else(|e| e.exit());
    Args {
        binary: matches.value_of("openvpn").unwrap().to_owned(),
        plugin_path: PathBuf::from(matches.value_of("plugin").unwrap()),
        config: PathBuf::from(matches.value_of("config").unwrap()),
        remotes: remotes,
        verbosity: matches.occurrences_of("verbose"),
    }
}

fn get_matches() -> ArgMatches<'static> {
    let app = create_app();
    app.clone().get_matches()
}

fn create_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("openvpn")
            .long("openvpn")
            .help("Specify what OpenVPN binary to run")
            .default_value("/usr/sbin/openvpn"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Specify what config file to start OpenVPN with")
            .default_value("./openvpn.conf"))
        .arg(Arg::with_name("remotes")
            .short("r")
            .long("remotes")
            .help("Configure what remote(s) to connect to. Accepts anything OpenVPN can use. \
                   Format: <address>:<port>")
            .takes_value(true)
            .multiple(true)
            .required(true))
        .arg(Arg::with_name("plugin")
            .long("plugin")
            .help("Path to talpid plugin")
            .default_value(DEFAULT_PLUGIN_PATH))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .multiple(true)
            .help("Sets the level of verbosity"))
}
