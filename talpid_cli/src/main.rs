// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate talpid_core;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate log;
extern crate env_logger;

use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use talpid_core::net::RemoteAddr;

use talpid_core::tunnel::{TunnelEvent, TunnelMonitor};

mod cli;


error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;
    let args = cli::parse_args_or_exit();
    main_loop(&args.remotes)
}

pub fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")
}

fn main_loop(remotes: &[RemoteAddr]) -> Result<()> {
    let mut remotes_iter = remotes.iter().cloned().cycle();
    let (monitor, rx) = create_tunnel_monitor()?;
    loop {
        monitor.start(remotes_iter.next().unwrap()).chain_err(|| "Unable to start OpenVPN")?;
        while let Ok(msg) = rx.recv() {
            match msg {
                TunnelEvent::Shutdown => {
                    println!("Monitored process exited");
                    break;
                }
                TunnelEvent::Up => println!("Tunnel UP"),
                TunnelEvent::Down => println!("Tunnel DOWN"),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn create_tunnel_monitor() -> Result<(TunnelMonitor, Receiver<TunnelEvent>)> {
    let (event_tx, event_rx) = mpsc::channel();
    let event_tx_mutex = Mutex::new(event_tx);
    let on_event = move |event: TunnelEvent| {
        event_tx_mutex.lock().unwrap().send(event).expect("Unable to send on tx_lock");
    };
    let monitor = TunnelMonitor::new(on_event).chain_err(|| "Unable to start OpenVPN monitor")?;
    Ok((monitor, event_rx))
}
