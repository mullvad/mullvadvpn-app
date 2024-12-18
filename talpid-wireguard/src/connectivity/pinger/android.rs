use std::net::Ipv4Addr;
use std::process::Stdio;
use std::time::Duration;

use tokio::io;
use tokio::process::{Child, Command};

/// Pinger errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to run `ping` process
    #[error("Failed to run ping command")]
    PingError(#[from] io::Error),

    /// ICMP timed out
    #[error("Ping timed out")]
    TimeoutError,
}

/// A pinger that sends ICMP requests without waiting for responses
pub struct Pinger {
    addr: Ipv4Addr,
    processes: Vec<Child>,
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
        self.processes.retain_mut(|child| {
            // retain non-terminated children
            matches!(child.try_wait(), Err(_) | Ok(None))
        });
    }
}

#[async_trait::async_trait]
impl super::Pinger for Pinger {
    // Send an ICMP packet without waiting for a reply
    async fn send_icmp(&mut self) -> Result<(), Error> {
        self.try_deplete_process_list();

        let child = ping_cmd(self.addr, Duration::from_secs(1)).map_err(Error::PingError)?;
        self.processes.push(child);
        Ok(())
    }

    async fn reset(&mut self) {
        self.processes.clear();
    }
}

fn ping_cmd(ip: Ipv4Addr, timeout: Duration) -> io::Result<Child> {
    let mut cmd = Command::new("ping");

    let timeout_secs = timeout.as_secs().to_string();
    let ip = ip.to_string();
    cmd.args(["-n", "-i", "1", "-w", &timeout_secs, &ip]);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    cmd.spawn()
}
