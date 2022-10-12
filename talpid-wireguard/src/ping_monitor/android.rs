use std::{io, net::Ipv4Addr};

/// Pinger errors
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to run `ping` process
    #[error(display = "Failed to run ping command")]
    PingError(#[error(source)] io::Error),

    /// ICMP timed out
    #[error(display = "Ping timed out")]
    TimeoutError,
}

/// A pinger that sends ICMP requests without waiting for responses
pub struct Pinger {
    addr: Ipv4Addr,
    processes: Vec<duct::Handle>,
}

impl Pinger {
    /// Creates a new pinger that will send ICMP requests only through the specified interface
    pub fn new(addr: Ipv4Addr) -> Result<Self, Error> {
        Ok(Self {
            processes: vec![],
            addr,
        })
    }

    fn try_deplete_process_list(&mut self) {
        self.processes.retain(|child| {
            match child.try_wait() {
                // child has terminated, doesn't have to be retained
                Ok(Some(_)) => false,
                _ => true,
            }
        });
    }
}

impl super::Pinger for Pinger {
    // Send an ICMP packet without waiting for a reply
    fn send_icmp(&mut self) -> Result<(), Error> {
        self.try_deplete_process_list();

        let cmd = ping_cmd(self.addr, 1);
        let handle = cmd.start().map_err(Error::PingError)?;
        self.processes.push(handle);
        Ok(())
    }

    fn reset(&mut self) {
        let processes = std::mem::replace(&mut self.processes, vec![]);
        for proc in processes {
            if proc
                .try_wait()
                .map(|maybe_stopped| maybe_stopped.is_none())
                .unwrap_or(false)
            {
                if let Err(err) = proc.kill() {
                    log::error!("Failed to kill ping process: {}", err);
                }
            }
        }
    }
}

impl Drop for Pinger {
    fn drop(&mut self) {
        for child in self.processes.iter_mut() {
            if let Err(e) = child.kill() {
                log::error!("Failed to kill ping process: {}", e);
            }
        }
    }
}

fn ping_cmd(ip: Ipv4Addr, timeout_secs: u16) -> duct::Expression {
    let timeout_secs = timeout_secs.to_string();
    let ip = ip.to_string();
    let args = ["-n", "-i", "1", "-w", &timeout_secs, &ip];

    duct::cmd("ping", args)
        .stdin_null()
        .stdout_null()
        .unchecked()
}
