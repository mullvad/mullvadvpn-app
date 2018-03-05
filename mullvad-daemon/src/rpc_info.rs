use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::result;

use mullvad_types::states::DaemonState;
use talpid_ipc::WsIpcClient;

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


/// Checks if there is another instance of the daemon running.
///
/// Tries to connect to another daemon and perform a simple RPC call. If it fails, assumes the
/// other daemon has stopped.
pub fn is_another_instance_running() -> bool {
    let rpc_file_exists = RPC_ADDRESS_FILE_PATH.as_path().exists();

    rpc_file_exists && other_daemon_responds()
}

/// Writes down the RPC connection info to some API to a file.
pub fn write(rpc_address: &str, shared_secret: &str) -> Result<()> {
    // Remove any existent RPC address file first
    let _ = remove();

    open_file(RPC_ADDRESS_FILE_PATH.as_path())
        .and_then(|mut file| write!(file, "{}\n{}\n", rpc_address, shared_secret))
        .chain_err(|| ErrorKind::WriteFailed(RPC_ADDRESS_FILE_PATH.to_owned()))?;

    debug!(
        "Wrote RPC connection info to {}",
        RPC_ADDRESS_FILE_PATH.to_string_lossy()
    );
    Ok(())
}

pub fn remove() -> Result<()> {
    fs::remove_file(RPC_ADDRESS_FILE_PATH.as_path())
        .chain_err(|| ErrorKind::RemoveFailed(RPC_ADDRESS_FILE_PATH.to_owned()))
}

fn other_daemon_responds() -> bool {
    if let Err(message) = call_other_daemon() {
        info!("{}; assuming it has stopped", message);
        false
    } else {
        true
    }
}

fn call_other_daemon() -> result::Result<(), String> {
    let method = "get_state";
    let args: [u8; 0] = [];
    let address = read_rpc_file().map_err(|_| "Failed to read RPC address file of other daemon")?;
    // TODO: Authenticate with server
    let mut rpc_client =
        WsIpcClient::new(address).map_err(|_| "Failed to connect to other daemon")?;
    let _: DaemonState = rpc_client
        .call(method, &args)
        .map_err(|_| "Failed to execute RPC call to other daemon")?;
    Ok(())
}

fn read_rpc_file() -> io::Result<String> {
    let file = File::open(RPC_ADDRESS_FILE_PATH.as_path())?;
    let mut reader = BufReader::new(file);
    let mut address = String::new();
    reader.read_line(&mut address)?;
    Ok(address)
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
