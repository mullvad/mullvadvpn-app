use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use mullvad_paths;

error_chain! {
    errors {
        UnknownFilePath {
            description("Failed to find path for RPC connection info file")
        }
        CreateDirFailed(path: PathBuf) {
            description("Failed to create directory for RPC connection info file")
            display(
                "Failed to create directory for RPC connection info file: {}",
                path.display(),
            )
        }
        WriteFailed(path: PathBuf) {
            description("Failed to write RPC connection info to file")
            display("Failed to write RPC connection info to {}", path.display())
        }
        RemoveFailed(path: PathBuf) {
            description("Failed to remove file")
            display("Failed to remove {}", path.display())
        }
    }
}

/// Writes down the RPC connection info to the RPC file.
pub fn write(rpc_address: &str, shared_secret: &str) -> Result<()> {
    // Avoids opening an existing file owned by another user and writing sensitive data to it.
    remove()?;

    let file_path = mullvad_paths::get_rpc_address_path().chain_err(|| ErrorKind::UnknownFilePath)?;

    if let Some(parent_dir) = file_path.parent() {
        fs::create_dir_all(parent_dir)
            .chain_err(|| ErrorKind::CreateDirFailed(parent_dir.to_owned()))?;
    }

    open_file(&file_path)
        .and_then(|mut file| write!(file, "{}\n{}\n", rpc_address, shared_secret))
        .chain_err(|| ErrorKind::WriteFailed(file_path.clone()))?;

    debug!("Wrote RPC connection info to {}", file_path.display());
    Ok(())
}

/// Removes the RPC file, if it exists.
pub fn remove() -> Result<()> {
    let file_path = mullvad_paths::get_rpc_address_path().chain_err(|| ErrorKind::UnknownFilePath)?;

    if let Err(error) = fs::remove_file(&file_path) {
        if error.kind() == io::ErrorKind::NotFound {
            // No previously existing file
            Ok(())
        } else {
            Err(error).chain_err(|| ErrorKind::RemoveFailed(file_path))
        }
    } else {
        Ok(())
    }
}

fn open_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    set_rpc_file_permissions(&file)?;
    Ok(file)
}

#[cfg(unix)]
fn set_rpc_file_permissions(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(PermissionsExt::from_mode(0o644))
}

#[cfg(windows)]
fn set_rpc_file_permissions(_file: &File) -> io::Result<()> {
    // TODO(linus): Lock permissions correctly on Windows.
    Ok(())
}
