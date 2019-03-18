use std::{net::IpAddr, thread, time};

error_chain! {
    errors {
        PingError{
            description("Failed to run ping")
        }

        TimeoutError {
            description("Ping timed out")
        }
    }
}

pub fn monitor_ping(ip: IpAddr, timeout_secs: u16, interface: &str) -> Result<()> {
    loop {
        let start = time::Instant::now();
        ping(ip, timeout_secs, &interface, false)?;
        if let Some(remaining) =
            time::Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed())
        {
            thread::sleep(remaining);
        }
    }
}

pub fn ping(
    ip: IpAddr,
    timeout_secs: u16,
    interface: &str,
    exit_on_first_reply: bool,
) -> Result<()> {
    let output = ping_cmd(ip, timeout_secs, interface, exit_on_first_reply)
        .run()
        .chain_err(|| ErrorKind::PingError)?;
    if !output.status.success() {
        bail!(ErrorKind::TimeoutError);
    }
    Ok(())
}

fn ping_cmd(
    ip: IpAddr,
    timeout_secs: u16,
    interface: &str,
    exit_on_first_reply: bool,
) -> duct::Expression {
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

    let timeout_secs = timeout_secs.to_string();
    let ip = ip.to_string();

    let mut args = vec![
        "-n",
        "-i",
        "1",
        &interface_flag,
        &interface,
        timeout_flag,
        &timeout_secs,
        &ip,
    ];
    if exit_on_first_reply {
        if cfg!(target_os = "macos") {
            args.push("-o");
        } else {
            args.extend_from_slice(&["-c", "1"])
        }
    }
    duct::cmd("ping", args)
        .stdin_null()
        .stdout_null()
        .unchecked()
}
