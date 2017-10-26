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

use error_chain::ChainedError;
use std::cmp::min;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Maximum number of bytes to read from each log file
const LOG_MAX_READ_BYTES: usize = 5 * 1024 * 1024;

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
        let log_paths = values_t!(collect_matches.values_of("logs"), String)
            .unwrap_or(Vec::new())
            .iter()
            .map(PathBuf::from)
            .collect();
        let output_path = value_t_or_exit!(collect_matches.value_of("output"), String);
        collect_report(log_paths, PathBuf::from(output_path))
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = value_t_or_exit!(send_matches.value_of("report"), String);
        let user_email = value_t!(send_matches.value_of("email"), String).unwrap_or(String::new());
        let user_message =
            value_t!(send_matches.value_of("message"), String).unwrap_or(String::new());
        send_problem_report(user_email, user_message, PathBuf::from(report_path))
    } else {
        unreachable!("No sub command given");
    }
}

fn collect_report(log_paths: Vec<PathBuf>, save_path: PathBuf) -> Result<()> {
    let mut problem_report = ProblemReport::default();
    for log_path in log_paths.into_iter() {
        problem_report.add_file_log(log_path);
    }
    write_problem_report(&save_path, problem_report)
        .chain_err(|| ErrorKind::WriteReportError(save_path.clone()))
}

fn send_problem_report(
    _user_email: String,
    _user_message: String,
    _report_path: PathBuf,
) -> Result<()> {
    // TODO: Implement submission to master
    Ok(())
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
    /// Attach file log to this report
    /// Unlike `try_add_file_log` this method uses the error description
    /// instead of log contents if error occurred when reading log file.
    fn add_file_log(&mut self, path: PathBuf) {
        if let Err(e) = self.try_add_file_log(&path)
            .chain_err(|| ErrorKind::ReadLogError(path.clone()))
        {
            self.logs.push((
                path.to_string_lossy().into_owned(),
                e.display_chain().to_string(),
            ));
        }
    }

    /// Try reading log from file source and attach it to this report
    fn try_add_file_log(&mut self, path: &Path) -> io::Result<()> {
        Ok(self.logs.push((
            path.to_string_lossy().into_owned(),
            Self::read_log_file(path, LOG_MAX_READ_BYTES)?,
        )))
    }

    /// Private helper to safely read the given number of bytes off the tail of UTF-8 log file
    /// and return it as a string
    fn read_log_file(path: &Path, max_bytes: usize) -> io::Result<String> {
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
