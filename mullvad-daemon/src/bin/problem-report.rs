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
extern crate regex;

extern crate mullvad_rpc;

use error_chain::ChainedError;
use regex::Regex;

use std::cmp::min;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum number of bytes to read from each log file
const LOG_MAX_READ_BYTES: usize = 1 * 1024 * 1024;
/// Fit two logs plus some system information in the report.
const REPORT_MAX_SIZE: usize = 2 * LOG_MAX_READ_BYTES + 16 * 1024;


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
        let redact_custom_strings = collect_matches
            .values_of_lossy("redact")
            .unwrap_or(Vec::new());
        let log_paths = collect_matches
            .values_of_os("logs")
            .map(|os_values| os_values.map(Path::new).collect())
            .unwrap_or(Vec::new());
        let output_path = Path::new(collect_matches.value_of_os("output").unwrap());
        collect_report(&log_paths, output_path, redact_custom_strings)
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = Path::new(send_matches.value_of_os("report").unwrap());
        let user_email = send_matches.value_of("email").unwrap_or("");
        let user_message = send_matches.value_of("message").unwrap_or("");
        send_problem_report(user_email, user_message, report_path)
    } else {
        unreachable!("No sub command given");
    }
}

fn collect_report(
    log_paths: &[&Path],
    output_path: &Path,
    redact_custom_strings: Vec<String>,
) -> Result<()> {
    let mut problem_report = ProblemReport::new(redact_custom_strings);
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
    redact_custom_strings: Vec<String>,
}

impl ProblemReport {
    /// Creates a new problem report with system information. Logs can be added with `add_log`.
    /// Logs will have all strings in `redact_custom_strings` removed from them.
    pub fn new(redact_custom_strings: Vec<String>) -> Self {
        ProblemReport {
            system_info: Self::collect_system_info(),
            logs: Vec::new(),
            redact_custom_strings,
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
        let mut out = self.redact_home_dir(input);

        out = self.redact_network_info(&out);

        self.redact_custom_strings(out)
    }

    fn redact_home_dir(&self, input: String) -> String {
        match env::home_dir() {
            Some(home) => input.replace(home.to_string_lossy().as_ref(), "~"),
            None => input,
        }
    }

    fn redact_network_info(&self, input: &str) -> String {
        let combined_pattern = format!(
            "\\b({}|{}|{})\\b",
            self.build_ipv4_regex(),
            self.build_ipv6_regex(),
            self.build_mac_regex()
        );
        let re = Regex::new(&combined_pattern).unwrap();

        re.replace_all(input, "[REDACTED]").to_string()
    }

    fn build_mac_regex(&self) -> String {
        let octet = "[[:xdigit:]]{2}"; // 0 - ff

        // five pairs of two hexadecimal chars followed by colon or dash
        // followed by a pair of hexadecimal chars
        format!("(?:{0}[:-]){{5}}({0})", octet)
    }

    fn build_ipv4_regex(&self) -> String {
        // regex adapted from  https://www.regular-expressions.info/ip.html

        let above_250 = "25[0-5]";
        let above_200 = "2[0-4][0-9]";
        let above_100 = "1[0-9][0-9]";

        // 100-119 | 120-126 | 128-129 | 130 - 199
        let above_100_not_127 = "1(?:[01][0-9]|2[0-6]|2[89]|[3-9][0-9])";

        let above_0 = "0?[0-9][0-9]?";

        // matches 0-255, except 127
        let first_octet = format!(
            "(?:{}|{}|{}|{})",
            above_250, above_200, above_100_not_127, above_0
        );

        // matches 0-255
        let ip_octet = format!("(?:{}|{}|{}|{})", above_250, above_200, above_100, above_0);

        format!("(?:{0}\\.{1}\\.{1}\\.{1})", first_octet, ip_octet)
    }

    fn build_ipv6_regex(&self) -> String {
        let hextet = "[[:xdigit:]]{1,4}"; // 0 - ffff

        // Matches 1-7 hextets followed by one or two colons
        // and one last hextet.
        //
        // This means that there are many
        // invalid IPv6 addresses that matches this. E.g.
        // all that has more than one instance of '::', but we
        // don't really care.
        let short = format!("({0}::?){{1,6}}(:{0}){{1,6}}", hextet);

        // Matches addresses without double colon. This is
        // a separate regex to make it easier to not match
        // on time
        let long = format!("({0}:){{7}}{0}", hextet);

        format!("(?:{})|(?:{})", short, long)
    }

    fn redact_custom_strings(&self, input: String) -> String {
        let mut out = input;
        for redact in &self.redact_custom_strings {
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
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_ipv4() {
        assert_redacts_ipv4("1.2.3.4");
        assert_redacts_ipv4("10.127.0.1");
        assert_redacts_ipv4("192.168.1.1");
        assert_redacts_ipv4("10.0.16.1");
        assert_redacts_ipv4("173.54.12.32");
        assert_redacts_ipv4("68.4.4.1");
    }

    fn assert_redacts_ipv4(input: &str) {
        let report = ProblemReport::new(vec![]);
        let actual = report.redact(format!("pre {} post", input));
        assert_eq!("pre [REDACTED] post", actual);
    }

    #[test]
    fn does_not_redact_localhost_ipv4() {
        let report = ProblemReport::new(vec![]);
        let res = report.redact("127.0.0.1".to_owned());
        assert_eq!("127.0.0.1", res);
    }

    #[test]
    fn redacts_ipv6() {
        assert_redacts_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert_redacts_ipv6("2001:db8:85a3:0:0:8a2e:370:7334");
        assert_redacts_ipv6("2001:db8:85a3::8a2e:370:7334");
        assert_redacts_ipv6("2001:db8:0:0:0:0:2:1");
        assert_redacts_ipv6("2001:db8::2:1");
        assert_redacts_ipv6("2001:db8:0000:1:1:1:1:1");
        assert_redacts_ipv6("2001:db8:0:1:1:1:1:1");
        assert_redacts_ipv6("2001:db8:0:0:1:0:0:1");
        assert_redacts_ipv6("2001:db8::1:0:0:1");
        assert_redacts_ipv6("0::0");
        assert_redacts_ipv6("0:0:0:0::1");
    }

    #[test]
    fn doesnt_redact_not_ipv6() {
        let report = ProblemReport::new(vec![]);
        let actual = report.redact(format!("[talpid_core::firewall]"));
        assert_eq!("[talpid_core::firewall]", actual);
    }

    fn assert_redacts_ipv6(input: &str) {
        let report = ProblemReport::new(vec![]);
        let actual = report.redact(format!("pre {} post", input));
        assert_eq!("pre [REDACTED] post", actual);
    }

    #[test]
    fn test_does_not_redact_localhost_ipv6() {
        let report = ProblemReport::new(vec![]);
        let res = report.redact("::1".to_owned());
        assert_eq!("::1", res);
    }

    #[test]
    fn test_does_not_redact_time() {
        let report = ProblemReport::new(vec![]);
        let res = report.redact("09:47:59".to_owned());
        assert_eq!("09:47:59", res);
    }
}
