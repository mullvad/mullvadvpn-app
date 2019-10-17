use std::{
    collections::HashSet,
    ffi::OsStr,
    fs, io,
    net::IpAddr,
    path::{Path, PathBuf},
};
use which::which;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to detect 'resolvconf' program")]
    NoResolvconf,

    #[error(display = "The resolvconf in PATH is just a symlink to systemd-resolved")]
    ResolvconfUsesResolved,

    #[error(display = "Failed to execute 'resolvconf' program")]
    RunResolvconf(#[error(source)] io::Error),

    #[error(display = "Using 'resolvconf' to add a record failed: {}", stderr)]
    AddRecordError { stderr: String },

    #[error(display = "Using 'resolvconf' to delete a record failed")]
    DeleteRecordError,
}

pub struct Resolvconf {
    record_names: HashSet<String>,
    resolvconf: PathBuf,
}

impl Resolvconf {
    pub fn new() -> Result<Self> {
        let resolvconf_path = which("resolvconf").map_err(|_| Error::NoResolvconf)?;
        if Self::resolvconf_is_resolved_symlink(&resolvconf_path) {
            return Err(Error::ResolvconfUsesResolved);
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
            .map_err(Error::RunResolvconf)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::AddRecordError { stderr });
        }

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
                .map_err(Error::RunResolvconf)?;

            if !output.status.success() {
                log::error!(
                    "Failed to delete 'resolvconf' record '{}':\n{}",
                    record_name,
                    String::from_utf8_lossy(&output.stderr)
                );
                result = Err(Error::DeleteRecordError);
            }
        }

        result
    }
}
