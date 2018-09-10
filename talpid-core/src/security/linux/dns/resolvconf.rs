use std::net::IpAddr;
use std::path::PathBuf;

use which::which;

error_chain! {
    errors {
        NoResolvconf {
            description("Failed to detect 'resolvconf' program")
        }
        RunResolvconf {
            description("Failed to execute 'resolvconf' program")
        }
        AddRecordError(stderr: String) {
            description("Using 'resolvconf' to add a record failed")
            display("Using 'resolvconf' to add a record failed: {}", stderr)
        }
        DeleteRecordError(stderr: String) {
            description("Using 'resolvconf' to delete a record failed")
            display("Using 'resolvconf' to delete a record failed: {}", stderr)
        }
    }
}

pub struct Resolvconf {
    record_name: Option<String>,
    resolvconf: PathBuf,
}

impl Resolvconf {
    pub fn new() -> Result<Self> {
        Ok(Resolvconf {
            record_name: None,
            resolvconf: which("resolvconf").map_err(|_| Error::from(ErrorKind::NoResolvconf))?,
        })
    }

    pub fn set_dns(&mut self, interface: &str, servers: Vec<IpAddr>) -> Result<()> {
        let record_name = format!("{}.mullvad", interface);
        let mut record_contents = String::new();

        for address in servers {
            record_contents.push_str("nameserver ");
            record_contents.push_str(&address.to_string());
            record_contents.push('\n');
        }

        let output = cmd!(&self.resolvconf, "-a", &record_name)
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
            let output = cmd!(&self.resolvconf, "-d", record_name)
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
