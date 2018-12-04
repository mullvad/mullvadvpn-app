use super::{CloseHandle, ErrorKind, Result, ResultExt};
use std::{net::IpAddr, thread, time};


pub fn spawn_ping_monitor(
    ip: IpAddr,
    timeout_secs: u16,
    interface: String,
    mut close_handle: Box<dyn CloseHandle>,
) {
    thread::spawn(move || loop {
        let start = time::Instant::now();
        if let Err(e) = ping(ip, timeout_secs, &interface) {
            close_handle.close_with_error(e);
            return;
        }
        let elapsed = time::Instant::now() - start;
        if let Some(remaining) = time::Duration::from_secs(timeout_secs.into()).checked_sub(elapsed)
        {
            thread::sleep(remaining);
        }
    });
}

pub fn ping(ip: IpAddr, timeout_secs: u16, interface: &str) -> Result<()> {
    let output = ping_cmd(ip, timeout_secs, interface)
        .run()
        .chain_err(|| ErrorKind::PingError)?;
    if !output.status.success() {
        bail!(ErrorKind::PingTimeoutError);
    }
    Ok(())
}

fn ping_cmd(ip: IpAddr, timeout_secs: u16, interface: &str) -> duct::Expression {
    let interface_flag = if cfg!(target_os = "linux") {
        "-I"
    } else {
        "-b"
    };
    let timeout_flag = if cfg!(target_os = "linux") {
        "-w"
    } else {
        "-t"
    };
    duct::cmd!(
        "ping",
        "-n",
        "-c",
        "1",
        &interface_flag,
        &interface,
        timeout_flag,
        &timeout_secs.to_string(),
        ip.to_string()
    )
    .stdin_null()
    .stdout_null()
    .unchecked()
}
