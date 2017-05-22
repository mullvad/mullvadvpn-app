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
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};

use talpid_core::process::openvpn::OpenVpnCommand;
use talpid_core::tunnel::openvpn::{OpenVpnEvent, OpenVpnMonitor};

mod cli;

use cli::Args;


error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;
    let args = cli::parse_args_or_exit();
    let command = create_openvpn_command(&args);
    main_loop(&command, args.plugin_path.as_path())
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

fn main_loop(command: &OpenVpnCommand, plugin_path: &Path) -> Result<()> {
    let (monitor, rx) = create_openvpn_monitor(plugin_path)?;
    loop {
        monitor.start(command.clone()).chain_err(|| "Unable to start OpenVPN")?;
        while let Ok(msg) = rx.recv() {
            match msg {
                OpenVpnEvent::Shutdown(result) => {
                    println!(
                        "Monitored process exited. clean: {}",
                        result.map(|s| s.success()).unwrap_or(false)
                    );
                    break;
                }
                OpenVpnEvent::PluginEvent(event, env) => {
                    println!("OpenVPN event:\nEvent: {:?}\nENV: {:?}", event, env);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn create_openvpn_monitor(plugin_path: &Path) -> Result<(OpenVpnMonitor, Receiver<OpenVpnEvent>)> {
    let (tx, rx) = mpsc::channel();
    let tx_mutex = Mutex::new(tx);
    let on_event = move |event: OpenVpnEvent| {
        let tx_lock = tx_mutex.lock().expect("Unable to lock tx_mutex");
        tx_lock.send(event).expect("Unable to send on tx_lock");
        println!("talpid_cli on_event fired and DONE")
    };
    let monitor = OpenVpnMonitor::new(on_event, plugin_path)
        .chain_err(|| "Unable to start OpenVPN monitor")?;
    Ok((monitor, rx))
}
