use std::fmt::Write;
use std::net::IpAddr;

use which::which;

error_chain! {
    errors {
        NoResolvconf {
            description("Failed to detect 'resolvconf' program")
        }
        RunResolvconf {
            description("Failed to execute resolvconf command")
        }
        AddRecordError(stderr: String) {
            description("Using resolvconf to add a record failed")
            display("Using resolvconf to add a record failed: {}", stderr)
        }
        DeleteRecordError(stderr: String) {
            description("Using resolvconf to delete a record failed")
            display("Using resolvconf to delete a record failed: {}", stderr)
        }
    }
}

pub struct Resolvconf {
    record_name: Option<String>,
}

impl Resolvconf {
    pub fn new() -> Result<Self> {
        which("resolvconf")
            .map(|_| Resolvconf { record_name: None })
            .map_err(|_| Error::from(ErrorKind::NoResolvconf))
    }

    pub fn set_dns(&mut self, interface: &str, servers: Vec<IpAddr>) -> Result<()> {
        let estimated_entry_length = "nameserver 255.255.255.255\n".len();

        let record_name = format!("{}.mullvad", interface);
        let mut record_contents = String::with_capacity(servers.len() * estimated_entry_length);

        for address in servers {
            write!(record_contents, "nameserver {}", address)
                .expect("Generated an invalid IP address string");
        }

        let output = cmd!("resolvconf", "-a", &record_name)
            .input(record_contents)
            .run()
            .chain_err(|| ErrorKind::RunResolvconf)?;

        ensure!(
            output.status.success(),
            ErrorKind::AddRecordError(String::from_utf8_lossy(&output.stderr).to_string())
        );

        self.record_name = Some(record_name);

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        if let Some(record_name) = self.record_name.take() {
            let output = cmd!("resolvconf", "-d", record_name)
                .run()
                .chain_err(|| ErrorKind::RunResolvconf)?;

            ensure!(
                output.status.success(),
                ErrorKind::DeleteRecordError(String::from_utf8_lossy(&output.stderr).to_string())
            )
        }

        Ok(())
    }
}
