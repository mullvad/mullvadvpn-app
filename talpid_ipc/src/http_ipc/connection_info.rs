use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// The path to the file where we write the connection infp
const IPC_CONNECTION_INFO_FILE: &'static str = "./.ipc_connection_info";

error_chain! {
    errors {
        OpenFileFailed(file_name: PathBuf) {
            description("Could not open file")
            display("Could not open {}", file_name.to_string_lossy())
        }
        WriteConnectionInfoFailed(file_name: PathBuf) {
            description("Could not write connection info file")
            display("Could not write to {}", file_name.to_string_lossy())
        }
    }
}

pub fn write(connection_info: &str) -> Result<()> {
    let file_location = PathBuf::from(IPC_CONNECTION_INFO_FILE);
    let mut file = open_file(&file_location)?;
    let res = file.write_all(connection_info.as_bytes())
        .chain_err(|| ErrorKind::WriteConnectionInfoFailed(file_location.clone()),);

    debug!(
        "Wrote IPC connection info ({}) to {}",
        connection_info,
        file_location.to_string_lossy()
    );

    res
}

fn open_file(file_name: &PathBuf) -> Result<File> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_name)
        .chain_err(|| ErrorKind::OpenFileFailed(file_name.to_owned()))
}
