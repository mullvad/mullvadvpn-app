use crate::mock_openvpn::run;

fn main() {
    run();
}

#[cfg(target_os = "android")]
mod mock_openvpn {
    pub fn run() {}
}

#[cfg(not(target_os = "android"))]
mod mock_openvpn {
    use mullvad_tests::{watch_event, PathWatcher};
    use std::{
        env,
        fs::{self, File},
        io::{self, Read, Write},
        path::PathBuf,
        sync::mpsc,
        thread,
        time::Duration,
    };

    const MAX_EVENT_TIME: Duration = Duration::from_secs(60);

    pub fn run() {
        let (file, path) = create_args_file();
        let (finished_tx, finished_rx) = mpsc::channel();
        let watcher = PathWatcher::watch(&path).expect("Failed to watch file for events");

        write_command_line(file);

        wait_thread(wait_for_stdin_to_be_closed, finished_tx.clone());
        wait_thread(
            move || wait_for_file_to_be_deleted(watcher, MAX_EVENT_TIME),
            finished_tx,
        );

        let _ = finished_rx.recv();
        let _ = fs::remove_file(path);
    }

    fn create_args_file() -> (File, PathBuf) {
        let path = PathBuf::from(
            env::var_os("MOCK_OPENVPN_ARGS_FILE")
                .expect("Missing mock OpenVPN arguments file path"),
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

    fn wait_for_file_to_be_deleted(mut watcher: PathWatcher, timeout: Duration) {
        let _ignore_event = watcher
            .set_timeout(timeout)
            .find(|&event| event == watch_event::REMOVE);
    }
}
