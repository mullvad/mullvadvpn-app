use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct DaemonInstance {
    process: Child,
    output: BufReader<ChildStdout>,
}

impl DaemonInstance {
    pub fn new() -> Self {
        let mut process = Command::new("target/debug/mullvad-daemon")
            .stdout(Stdio::piped())
            .current_dir("..")
            .args(&["--resource-dir", "dist-assets"])
            .spawn()
            .expect("failed to start daemon");
        let output = BufReader::new(process.stdout.take().expect("missing daemon stdout"));

        DaemonInstance { process, output }
    }

    pub fn output(&mut self) -> String {
        let mut bytes = Vec::new();
        self.output
            .read_to_end(&mut bytes)
            .expect("failed to read daemon stdout");
        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn expect(&mut self, pattern: &'static str, timeout: Duration) {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            thread::sleep(timeout);
            if rx.try_recv().is_err() {
                panic!("timeout while searching for {:?}", pattern);
            }
        });

        self.search_in_stdout(pattern);
        let _ = tx.send(());
    }

    fn search_in_stdout(&mut self, pattern: &str) {
        let mut line = String::new();

        while !line.contains(pattern) {
            self.output
                .read_line(&mut line)
                .expect("failed to read line from daemon stdout");
            println!("{:?}", line);
        }
    }
}

impl Drop for DaemonInstance {
    fn drop(&mut self) {
        self.process.kill().expect("failed to kill daemon process");
    }
}
