extern crate notify;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

fn main() {
    let (file, path) = create_mock_file();

    write_command_line(file);
    wait_for_file_to_be_deleted(path);
}

fn create_mock_file() -> (File, PathBuf) {
    let path = PathBuf::from(
        env::var_os("MOCK_OPENVPN_FILE").expect("Missing mock OpenVPN auxiliary file path"),
    );
    let file = File::create(&path).expect("Failed to create mock OpenVPN auxiliary file");

    (file, path)
}

fn write_command_line(mut file: File) {
    for argument in env::args() {
        let escaped_argument = argument
            .replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace("\r", "\\r");

        writeln!(file, "{}", escaped_argument).expect("Failed to write argument to output file");
    }
}

fn wait_for_file_to_be_deleted<P: AsRef<Path>>(file: P) {
    let file = file.as_ref();
    let (tx, rx) = mpsc::channel();

    let mut watcher = raw_watcher(tx).expect(&format!(
        "Failed to create file watcher for \"{}\"",
        file.display()
    ));

    watcher
        .watch(&file, RecursiveMode::NonRecursive)
        .expect(&format!("Failed to watch file: {}", file.display()));

    for event in rx {
        if let RawEvent {
            path: Some(path),
            op: Ok(op),
            ..
        } = event
        {
            if op.contains(notify::op::REMOVE) && path == file {
                break;
            }
        }
    }
}
