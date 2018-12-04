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

pub fn spawn_ping_monitor<F: FnOnce() + Send + 'static>(
    ip: IpAddr,
    timeout_secs: u16,
    interface: String,
    on_fail: F,
) {
    thread::spawn(move || loop {
        let start = time::Instant::now();
        if let Err(e) = ping(ip, timeout_secs, &interface) {
            log::debug!("ping failed - {}", e);
            on_fail();
            return;
        }
        if let Some(remaining) =
            time::Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed())
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
        bail!(ErrorKind::TimeoutError);
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
