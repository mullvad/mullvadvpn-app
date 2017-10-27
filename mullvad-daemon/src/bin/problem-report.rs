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
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

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
    let app = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(
            clap::SubCommand::with_name("collect")
                .about("Collect problem report")
                .arg(
                    clap::Arg::with_name("output")
                        .long("output")
                        .short("o")
                        .takes_value(true)
                        .help("The destination path for saving the collected report.")
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("logs")
                        .help("The paths to log files to include in the problem report.")
                        .multiple(true)
                        .takes_value(true)
                        .required(false),
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
        let log_paths = collect_matches
            .values_of_os("logs")
            .map(|os_values| os_values.map(Path::new).collect())
            .unwrap_or(Vec::new());
        let output_path = Path::new(collect_matches.value_of_os("output").unwrap());
        collect_report(&log_paths, output_path)
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = Path::new(send_matches.value_of_os("report").unwrap());
        let user_email = send_matches.value_of("email").unwrap_or("");
        let user_message = send_matches.value_of("message").unwrap_or("");
        send_problem_report(user_email, user_message, report_path)
    } else {
        unreachable!("No sub command given");
    }
}

fn collect_report(log_paths: &[&Path], output_path: &Path) -> Result<()> {
    let mut problem_report = ProblemReport::default();
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

#[derive(Debug, Default)]
struct ProblemReport {
    logs: Vec<(String, String)>,
}

impl ProblemReport {
    /// Attach file log to this report. This method uses the error chain instead of log
    /// contents if error occurred when reading log file.
    fn add_log(&mut self, path: &Path) {
        let content = read_file_lossy(path, LOG_MAX_READ_BYTES)
            .chain_err(|| ErrorKind::ReadLogError(path.to_path_buf()))
            .unwrap_or_else(|e| e.display_chain().to_string());
        self.logs
            .push((path.to_string_lossy().into_owned(), content));
    }
}

impl fmt::Display for ProblemReport {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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
