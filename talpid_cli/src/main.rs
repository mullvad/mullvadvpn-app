// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate talpid_core;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate log;
extern crate env_logger;

use std::path::Path;
use std::sync::mpsc::{self, Receiver};

use talpid_core::process::openvpn::{OpenVpnCommand, OpenVpnEvent, OpenVpnMonitor};

mod cli;

use cli::Args;


error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;
    let args = cli::parse_args_or_exit();
    let command = create_openvpn_command(&args);
    main_loop(command, args.plugin_path.as_path())
}

pub fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")
}

fn create_openvpn_command(args: &Args) -> OpenVpnCommand {
    let mut command = OpenVpnCommand::new(&args.binary);
    command
        .config(&args.config)
        .remotes(&args.remotes[..])
        .unwrap();
    command
}

fn main_loop(command: OpenVpnCommand, plugin_path: &Path) -> Result<()> {
    loop {
        let (_monitor, rx) = start_monitor(command.clone(), plugin_path)?;
        while let Ok(msg) = rx.recv() {
            match msg {
                OpenVpnEvent::Shutdown(result) => {
                    println!(
                        "Monitored process exited. clean: {}",
                        result.map(|s| s.success()).unwrap_or(false)
                    );
                    break;
                }
                OpenVpnEvent::PluginEvent(Ok((event, env))) => {
                    println!("OpenVPN event:\nEvent: {:?}\nENV: {:?}", event, env);
                }
                OpenVpnEvent::PluginEvent(Err(e)) => println!("Read error from plugin: {:?}", e),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn start_monitor(command: OpenVpnCommand,
                 plugin_path: &Path)
                 -> Result<(OpenVpnMonitor, Receiver<OpenVpnEvent>)> {
    let (tx, rx) = mpsc::channel();
    let listener = move |event: OpenVpnEvent| tx.send(event).unwrap();
    OpenVpnMonitor::start(command, plugin_path, listener)
        .map(|m| (m, rx))
        .chain_err(|| "Unable to start OpenVPN")
}
