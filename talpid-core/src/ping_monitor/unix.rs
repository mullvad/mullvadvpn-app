#[allow(dead_code)]
// TODO: remove the lint exemption above when ping monitor is used
use std::{
    io,
    net::Ipv4Addr,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to run ping command")]
    PingError(#[error(cause)] io::Error),

    #[error(display = "Ping timed out")]
    TimeoutError,
}

pub fn monitor_ping(
    ip: Ipv4Addr,
    timeout_secs: u16,
    interface: &str,
    close_receiver: mpsc::Receiver<()>,
) -> Result<(), Error> {
    while let Err(mpsc::TryRecvError::Empty) = close_receiver.try_recv() {
        let start = Instant::now();
        internal_ping(ip, timeout_secs, &interface, false)?;
        if let Some(remaining) =
            Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed())
        {
            thread::sleep(remaining);
        }
    }

    Ok(())
}

pub fn ping(ip: Ipv4Addr, timeout_secs: u16, interface: &str) -> Result<(), Error> {
    internal_ping(ip, timeout_secs, interface, true)
}

fn internal_ping(
    ip: Ipv4Addr,
    timeout_secs: u16,
    interface: &str,
    exit_on_first_reply: bool,
) -> Result<(), Error> {
    let output = ping_cmd(ip, timeout_secs, interface, exit_on_first_reply)
        .run()
        .map_err(Error::PingError)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::TimeoutError)
    }
}

fn ping_cmd(
    ip: Ipv4Addr,
    timeout_secs: u16,
    interface: &str,
    exit_on_first_reply: bool,
) -> duct::Expression {
    let mut args = vec!["-n", "-i", "1"];

    let timeout_flag = if cfg!(target_os = "linux") || cfg!(target_os = "android") {
        "-w"
    } else {
        "-t"
    };
    let timeout_secs = timeout_secs.to_string();

    args.extend_from_slice(&[timeout_flag, &timeout_secs]);

    let interface_flag = if cfg!(target_os = "linux") {
        Some("-I")
    } else if cfg!(target_os = "macos") {
        Some("-b")
    } else {
        None
    };

    if let Some(interface_flag) = interface_flag {
        args.extend_from_slice(&[interface_flag, interface]);
    }

    if exit_on_first_reply {
        if cfg!(target_os = "macos") {
            args.push("-o");
        } else {
            args.extend_from_slice(&["-c", "1"])
        }
    }

    let ip = ip.to_string();
    args.push(&ip);

    duct::cmd("ping", args)
        .stdin_null()
        .stdout_null()
        .unchecked()
}
