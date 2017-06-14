use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

error_chain! {
    errors {
        WriteFailed(path: PathBuf) {
            description("Failed to write RCP address to file")
            display("Failed to write RPC address to {}", path.to_string_lossy())
        }
        RemoveFailed(path: PathBuf) {
            description("Failed to remove file")
            display("Failed to remove {}", path.to_string_lossy())
        }
    }
}

lazy_static! {
    /// The path to the file where we write the RPC address
    static ref RPC_ADDRESS_FILE_PATH: &'static Path = Path::new("./.rpc_address");
}

/// Writes down the RPC address to some API to a file.
pub fn write(rpc_address: &str) -> Result<()> {
    open_file(*RPC_ADDRESS_FILE_PATH)
        .and_then(|mut file| file.write_all(rpc_address.as_bytes()))
        .chain_err(|| ErrorKind::WriteFailed(RPC_ADDRESS_FILE_PATH.to_owned()))?;

    debug!(
        "Wrote RPC address to {}",
        RPC_ADDRESS_FILE_PATH.to_string_lossy()
    );
    Ok(())
}

pub fn remove() -> Result<()> {
    fs::remove_file(*RPC_ADDRESS_FILE_PATH)
        .chain_err(|| ErrorKind::RemoveFailed(RPC_ADDRESS_FILE_PATH.to_owned()))
}

fn open_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
}
