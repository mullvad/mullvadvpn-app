extern crate notify;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};

fn main() {
    let (file, path) = create_args_file();
    let path_to_wait_for = path;
    let path_to_remove = path_to_wait_for.clone();
    let (finished_tx, finished_rx) = mpsc::channel();

    write_command_line(file);

    wait_thread(wait_for_stdin_to_be_closed, finished_tx.clone());
    wait_thread(
        move || wait_for_file_to_be_deleted(path_to_wait_for),
        finished_tx,
    );

    let _ = finished_rx.recv();
    let _ = fs::remove_file(path_to_remove);
}

fn create_args_file() -> (File, PathBuf) {
    let path = PathBuf::from(
        env::var_os("MOCK_OPENVPN_ARGS_FILE").expect("Missing mock OpenVPN arguments file path"),
    );
    let file = File::create(&path).expect("Failed to create mock OpenVPN arguments file");

    (file, path)
}

fn write_command_line(mut file: File) {
    for argument in env::args() {
        let escaped_argument = argument
            .replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace("\r", "\\r");

        writeln!(file, "{}", escaped_argument).expect("Failed to write argument to file");
    }
}

fn wait_thread<F>(function: F, finished_tx: mpsc::Sender<()>)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(move || {
        function();
        let _ = finished_tx.send(());
    });
}

fn wait_for_stdin_to_be_closed() {
    let _ignore_bytes = io::stdin().bytes().last();
}

fn wait_for_file_to_be_deleted<P: AsRef<Path>>(file: P) {
    let file = file.as_ref();
    let (tx, rx) = mpsc::channel();

    if let Ok(mut watcher) = raw_watcher(tx) {
        if watcher.watch(&file, RecursiveMode::NonRecursive).is_ok() {
            for event in rx {
                if let RawEvent { op: Ok(op), .. } = event {
                    if op.contains(notify::op::REMOVE) {
                        break;
                    }
                }
            }
        }
    }
}
