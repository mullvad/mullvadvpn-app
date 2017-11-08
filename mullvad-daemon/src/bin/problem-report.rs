//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;

extern crate mullvad_rpc;

use error_chain::ChainedError;

use std::cmp::min;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum number of bytes to read from each log file
const LOG_MAX_READ_BYTES: usize = 5 * 1024 * 1024;
/// Maximum number of bytes allowed in a report.
const REPORT_MAX_SIZE: usize = 4 * LOG_MAX_READ_BYTES;


/// Field delimeter in generated problem report
const LOG_DELIMITER: &'static str = "====================";

error_chain!{
    errors {
        WriteReportError(path: PathBuf) {
            description("Error writing the problem report file")
            display("Error writing the problem report file: {}", path.to_string_lossy())
        }
        ReadLogError(path: PathBuf) {
            description("Error reading the contents of log file")
            display("Error reading the contents of log file: {}", path.to_string_lossy())
        }
        RpcError {
            description("Error during RPC call")
        }
    }
}

quick_main!(run);

fn run() -> Result<()> {
    let app = clap::App::new("problem-report")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Mullvad VPN problem report tool. Collects logs and send them to Mullvad support.")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(
            clap::SubCommand::with_name("collect")
                .about("Collect problem report")
                .arg(
                    clap::Arg::with_name("output")
                        .help("The destination path for saving the collected report.")
                        .long("output")
                        .short("o")
                        .value_name("PATH")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("logs")
                        .help("The paths to log files to include in the problem report.")
                        .multiple(true)
                        .value_name("LOG PATHS")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::with_name("redact")
                        .help("List of words and expressions to remove from the report")
                        .long("redact")
                        .value_name("PHRASE")
                        .multiple(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("send")
                .about("Send collected problem report")
                .arg(
                    clap::Arg::with_name("report")
                        .long("report")
                        .short("r")
                        .help("The path to previously collected report file.")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("email")
                        .long("email")
                        .short("e")
                        .help("Reporter's email")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::with_name("message")
                        .long("message")
                        .short("m")
                        .help("Reporter's message")
                        .takes_value(true)
                        .required(false),
                ),
        );

    let matches = app.get_matches();

    if let Some(collect_matches) = matches.subcommand_matches("collect") {
        let redacts = collect_matches
            .values_of_lossy("redact")
            .unwrap_or(Vec::new());
        let log_paths = collect_matches
            .values_of_os("logs")
            .map(|os_values| os_values.map(Path::new).collect())
            .unwrap_or(Vec::new());
        let output_path = Path::new(collect_matches.value_of_os("output").unwrap());
        collect_report(&log_paths, output_path, redacts)
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = Path::new(send_matches.value_of_os("report").unwrap());
        let user_email = send_matches.value_of("email").unwrap_or("");
        let user_message = send_matches.value_of("message").unwrap_or("");
        send_problem_report(user_email, user_message, report_path)
    } else {
        unreachable!("No sub command given");
    }
}

fn collect_report(log_paths: &[&Path], output_path: &Path, redacts: Vec<String>) -> Result<()> {
    let mut problem_report = ProblemReport::new(redacts);
    for log_path in log_paths {
        problem_report.add_log(log_path);
    }
    write_problem_report(&output_path, problem_report)
        .chain_err(|| ErrorKind::WriteReportError(output_path.to_path_buf()))
}

fn send_problem_report(user_email: &str, user_message: &str, report_path: &Path) -> Result<()> {
    let report_content = read_file_lossy(report_path, REPORT_MAX_SIZE)
        .chain_err(|| ErrorKind::ReadLogError(report_path.to_path_buf()))?;
    let mut rpc_client =
        mullvad_rpc::ProblemReportProxy::connect().chain_err(|| ErrorKind::RpcError)?;
    rpc_client
        .problem_report(user_email, user_message, &report_content)
        .call()
        .chain_err(|| ErrorKind::RpcError)
}

fn write_problem_report(path: &Path, problem_report: ProblemReport) -> io::Result<()> {
    let mut file = File::create(path)?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)?;
    file.write(problem_report.to_string().as_bytes())?;
    Ok(())
}


#[derive(Debug)]
struct ProblemReport {
    system_info: Vec<String>,
    logs: Vec<(String, String)>,
    redacts: Vec<String>,
}

impl ProblemReport {
    /// Creates a new problem report with system information. Logs can be added with `add_log`.
    /// Logs will have all strings in `redacts` removed from them.
    pub fn new(redacts: Vec<String>) -> Self {
        ProblemReport {
            system_info: Self::collect_system_info(),
            logs: Vec::new(),
            redacts,
        }
    }

    fn collect_system_info() -> Vec<String> {
        vec![
            format!("Mullvad daemon: {}", daemon_version()),
            format!("OS: {}", os_version()),
        ]
    }

    /// Attach file log to this report. This method uses the error chain instead of log
    /// contents if error occurred when reading log file.
    pub fn add_log(&mut self, path: &Path) {
        let content = self.redact(
            read_file_lossy(path, LOG_MAX_READ_BYTES)
                .chain_err(|| ErrorKind::ReadLogError(path.to_path_buf()))
                .unwrap_or_else(|e| e.display_chain().to_string()),
        );
        let path = self.redact(path.to_string_lossy().into_owned());
        self.logs.push((path, content));
    }

    fn redact(&self, input: String) -> String {
        let mut out = match env::home_dir() {
            Some(home) => input.replace(home.to_string_lossy().as_ref(), "~"),
            None => input,
        };
        for redact in &self.redacts {
            out = out.replace(redact, "[REDACTED]")
        }
        out
    }
}

impl fmt::Display for ProblemReport {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "System information:")?;
        for system_info in &self.system_info {
            writeln!(fmt, "{}", system_info)?;
        }
        writeln!(fmt, "")?;
        for &(ref label, ref content) in &self.logs {
            writeln!(fmt, "{}", LOG_DELIMITER)?;
            writeln!(fmt, "Log: {}", label)?;
            writeln!(fmt, "{}", LOG_DELIMITER)?;
            fmt.write_str(content)?;
            writeln!(fmt)?;
        }
        Ok(())
    }
}

/// Helper to lossily read a file to a `String`. If the file size exceeds the given `max_bytes`,
/// only the last `max_bytes` bytes of the file are read.
fn read_file_lossy(path: &Path, max_bytes: usize) -> io::Result<String> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    if file_size > max_bytes as u64 {
        file.seek(SeekFrom::Start(file_size - max_bytes as u64))?;
    }

    let capacity = min(file_size, max_bytes as u64) as usize;
    let mut buffer = Vec::with_capacity(capacity);
    file.take(max_bytes as u64).read_to_end(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn daemon_version() -> String {
    format!(
        "v{} {}",
        env!("CARGO_PKG_VERSION"),
        include_str!(concat!(env!("OUT_DIR"), "/git-commit-info.txt"))
    )
}

#[cfg(target_os = "linux")]
fn os_version() -> String {
    format!(
        "Linux, {}",
        command_stdout_lossy("lsb_release", &["-ds"])
            .unwrap_or(String::from("[Failed to get LSB release]"))
    )
}

#[cfg(target_os = "macos")]
fn os_version() -> String {
    format!(
        "macOS {}",
        command_stdout_lossy("sw_vers", &["-productVersion"])
            .unwrap_or(String::from("[Failed to detect version]"))
    )
}

#[cfg(windows)]
fn os_version() -> String {
    String::from("Windows")
}

/// Helper for getting stdout of some command as a String. Ignores the exit code of the command.
fn command_stdout_lossy(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        })
        .ok()
}
