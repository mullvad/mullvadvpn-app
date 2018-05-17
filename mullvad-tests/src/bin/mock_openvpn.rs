extern crate mullvad_tests;
extern crate notify;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use mullvad_tests::MOCK_OPENVPN_COMMAND_LINE_FILE;
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

fn main() {
    let (file, path) = create_mock_file();

    write_command_line(file);
    wait_for_file_to_be_deleted(path);
}

fn create_mock_file() -> (File, PathBuf) {
    let path = find_test_directory().join(MOCK_OPENVPN_COMMAND_LINE_FILE);
    let file = File::create(&path).expect("Failed to create mock OpenVPN auxiliary file");

    (file, path)
}

fn find_test_directory() -> PathBuf {
    let current_dir = env::current_exe().expect("Failed to discover executable directory");

    {
        let mut dir: &Path = &current_dir;

        if !is_test_directory(dir) {
            while let Some(parent) = dir.parent() {
                dir = parent;

                if is_test_directory(dir) {
                    return dir.to_path_buf();
                }
            }
        }
    }

    current_dir
}

fn is_test_directory(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        name.to_string_lossy().contains("mullvad-daemon-test")
    } else {
        false
    }
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
