use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};
use which::which;

error_chain! {
    errors {
        NoResolvconf {
            description("Failed to detect 'resolvconf' program")
        }
        ResolvconfUsesResolved {
            description("The existing resolvconf binary is just a symlink to systemd-resolved")
        }
        RunResolvconf {
            description("Failed to execute 'resolvconf' program")
        }
        AddRecordError(stderr: String) {
            description("Using 'resolvconf' to add a record failed")
            display("Using 'resolvconf' to add a record failed: {}", stderr)
        }
        DeleteRecordError {
            description("Using 'resolvconf' to delete a record failed")
        }
    }
}

pub struct Resolvconf {
    record_names: HashSet<String>,
    resolvconf: PathBuf,
}

impl Resolvconf {
    pub fn new() -> Result<Self> {
        let resolvconf_path =
            which("resolvconf").map_err(|_| Error::from(ErrorKind::NoResolvconf))?;
        if Self::resolvconf_is_resolved_symlink(&resolvconf_path) {
            bail!(ErrorKind::ResolvconfUsesResolved);
        }
        Ok(Resolvconf {
            record_names: HashSet::new(),
            resolvconf: resolvconf_path,
        })
    }

    fn resolvconf_is_resolved_symlink(resolvconf_path: &Path) -> bool {
        fs::read_link(resolvconf_path)
            .map(|resolvconf_target| {
                resolvconf_target.file_name() == Some(OsStr::new("resolvectl"))
            })
            .unwrap_or_else(|_| false)
    }

    pub fn set_dns(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        let record_name = format!("{}.mullvad", interface);
        let mut record_contents = String::new();

        for address in servers {
            record_contents.push_str("nameserver ");
            record_contents.push_str(&address.to_string());
            record_contents.push('\n');
        }

        let output = duct::cmd!(&self.resolvconf, "-a", &record_name)
            .input(record_contents)
            .stderr_capture()
            .unchecked()
            .run()
            .chain_err(|| ErrorKind::RunResolvconf)?;

        ensure!(
            output.status.success(),
            ErrorKind::AddRecordError(String::from_utf8_lossy(&output.stderr).to_string())
        );

        self.record_names.insert(record_name);

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        let mut result = Ok(());

        for record_name in self.record_names.drain() {
            let output = duct::cmd!(&self.resolvconf, "-d", &record_name)
                .stderr_capture()
                .unchecked()
                .run()
                .chain_err(|| ErrorKind::RunResolvconf)?;

            if !output.status.success() {
                log::error!(
                    "Failed to delete 'resolvconf' record '{}':\n{}",
                    record_name,
                    String::from_utf8_lossy(&output.stderr)
                );
                result = Err(Error::from(ErrorKind::DeleteRecordError));
            }
        }

        result
    }
}
