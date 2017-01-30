extern crate talpid_core;
#[macro_use]
extern crate clap;

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use talpid_core::process::OpenVpnCommand;
use talpid_core::process::monitor::{ChildMonitor, TransitionResult, ChildSpawner};

mod cli;

use cli::Args;

/// Macro for printing to stderr. Will simply do nothing if the printing fails for some reason.
macro_rules! eprintln {
    ($($arg:tt)*) => (
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($arg)* );
    )
}

fn main() {
    let args = cli::parse_args_or_exit();

    let command = create_openvpn_command(&args);
    let monitor = ChildMonitor::new(command);
    if let Err(e) = main_loop(monitor) {
        eprintln!("OpenVPN failed: {}", e);
    }
}

fn create_openvpn_command(args: &Args) -> OpenVpnCommand {
    let mut command = OpenVpnCommand::new(&args.binary);
    command.config(&args.config)
        .remotes(&args.remotes[..])
        .unwrap()
        .pipe_output(args.verbosity > 0);

    command
}

fn main_loop<S>(mut monitor: ChildMonitor<S>) -> TransitionResult<()>
    where S: ChildSpawner
{
    loop {
        let rx = start_monitor(&mut monitor)?;
        let clean_exit = rx.recv().unwrap();
        println!("Monitored process exited. clean: {}", clean_exit);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn start_monitor<S>(monitor: &mut ChildMonitor<S>) -> TransitionResult<Receiver<bool>>
    where S: ChildSpawner
{
    let (tx, rx) = mpsc::channel();
    let callback = move |clean| tx.send(clean).unwrap();
    monitor.start(callback).map(|(stdout, stderr)| {
        stdout.map(|stream| pass_io(stream, io::stdout()));
        stderr.map(|stream| pass_io(stream, io::stderr()));
        rx
    })
}

fn pass_io<I, O>(mut input: I, mut output: O)
    where I: Read + Send + 'static,
          O: Write + Send + 'static
{
    thread::spawn(move || { io::copy(&mut input, &mut output).unwrap(); });
}
