use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

error_chain! {
    errors {
        WriteFailed(path: PathBuf) {
            description("Failed to write RCP connection info to file")
            display("Failed to write RPC connection info to {}", path.to_string_lossy())
        }
        RemoveFailed(path: PathBuf) {
            description("Failed to remove file")
            display("Failed to remove {}", path.to_string_lossy())
        }
    }
}

#[cfg(unix)]
lazy_static! {
    /// The path to the file where we write the RPC connection info
    static ref RPC_ADDRESS_FILE_PATH: PathBuf = Path::new("/tmp").join(".mullvad_rpc_address");
}

#[cfg(not(unix))]
lazy_static! {
    /// The path to the file where we write the RPC connection info
    static ref RPC_ADDRESS_FILE_PATH: PathBuf = ::std::env::temp_dir().join(".mullvad_rpc_address");
}


/// Reads the address of the RPC connection from the RPC info file.
pub fn read() -> io::Result<String> {
    let file = File::open(RPC_ADDRESS_FILE_PATH.as_path())?;
    let mut reader = BufReader::new(file);
    let mut address = String::new();
    reader.read_line(&mut address)?;
    Ok(address)
}

/// Writes down the RPC connection info to some API to a file.
pub fn write(rpc_address: &str, shared_secret: &str) -> Result<()> {
    // Avoids opening an existing file owned by another user and writing sensitive data to it.
    remove()?;

    open_file(RPC_ADDRESS_FILE_PATH.as_path())
        .and_then(|mut file| write!(file, "{}\n{}\n", rpc_address, shared_secret))
        .chain_err(|| ErrorKind::WriteFailed(RPC_ADDRESS_FILE_PATH.to_owned()))?;

    debug!(
        "Wrote RPC connection info to {}",
        RPC_ADDRESS_FILE_PATH.to_string_lossy()
    );
    Ok(())
}

/// Removes the RPC file, if it exists.
pub fn remove() -> Result<()> {
    if let Err(error) = fs::remove_file(RPC_ADDRESS_FILE_PATH.as_path()) {
        if error.kind() == io::ErrorKind::NotFound {
            // No previously existing file
            Ok(())
        } else {
            Err(error).chain_err(|| ErrorKind::RemoveFailed(RPC_ADDRESS_FILE_PATH.to_owned()))
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
