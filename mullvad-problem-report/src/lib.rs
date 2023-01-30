#![deny(rust_2018_idioms)]

use lazy_static::lazy_static;
use mullvad_api::proxy::ApiConnectionMode;
use regex::Regex;
use std::{
    borrow::Cow,
    cmp::min,
    collections::{BTreeMap, HashSet},
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use talpid_types::ErrorExt;

pub mod metadata;

/// Maximum number of bytes to read from each log file
const LOG_MAX_READ_BYTES: usize = 128 * 1024;
const EXTRA_BYTES: usize = 32 * 1024;
/// Fit five logs plus some system information in the report.
const REPORT_MAX_SIZE: usize = (5 * LOG_MAX_READ_BYTES) + EXTRA_BYTES;

/// Field delimeter in generated problem report
const LOG_DELIMITER: &str = "====================";

/// Line separator character sequence
#[cfg(not(windows))]
const LINE_SEPARATOR: &str = "\n";

#[cfg(windows)]
const LINE_SEPARATOR: &str = "\r\n";

const MAX_SEND_ATTEMPTS: usize = 3;

/// Custom macro to write a line to an output formatter that uses platform-specific newline
/// character sequences.
macro_rules! write_line {
    ($fmt:expr $(,)*) => { write!($fmt, "{}", LINE_SEPARATOR) };
    ($fmt:expr, $pattern:expr $(, $arg:expr)* $(,)*) => {
        write!($fmt, $pattern, $( $arg ),*)
            .and_then(|_| write!($fmt, "{}", LINE_SEPARATOR))
    };
}

/// These are critical errors that can happen when using the tool, that stops
/// it from working. Meaning it will print the error and exit.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to write the problem report to {}", path)]
    WriteReportError {
        path: String,
        #[error(source)]
        source: io::Error,
    },

    #[error(display = "Failed to read the problem report at {}", path)]
    ReadProblemReportError {
        path: String,
        #[error(source)]
        source: io::Error,
    },

    #[error(display = "Unable to create REST client")]
    CreateRpcClientError(#[error(source)] mullvad_api::Error),

    #[error(display = "Failed to send problem report")]
    SendProblemReportError(#[error(source)] mullvad_api::rest::Error),

    #[error(display = "Failed to send problem report {} times", MAX_SEND_ATTEMPTS)]
    SendFailedTooManyTimes,

    #[error(display = "Unable to spawn Tokio runtime")]
    CreateRuntime(#[error(source)] io::Error),

    #[error(display = "Unable to find cache directory")]
    ObtainCacheDirectory(#[error(source)] mullvad_paths::Error),
}

/// These are errors that can happen during problem report collection.
/// They are not critical, but they will be added inside the problem report,
/// instead of whatever content was supposed to be there.
#[derive(err_derive::Error, Debug)]
pub enum LogError {
    #[error(display = "Unable to get log directory")]
    GetLogDir(#[error(source)] mullvad_paths::Error),

    #[error(display = "Failed to list the files in the log directory: {}", path)]
    ListLogDir {
        path: String,
        #[error(source)]
        source: io::Error,
    },

    #[error(display = "Error reading the contents of log file: {}", path)]
    ReadLogError { path: String },

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[error(display = "No home directory for current user")]
    NoHomeDir,

    #[cfg(target_os = "windows")]
    #[error(display = "Missing %LOCALAPPDATA% environment variable")]
    NoLocalAppDataDir,
}

pub fn collect_report(
    extra_logs: &[&Path],
    output_path: &Path,
    redact_custom_strings: Vec<String>,
    #[cfg(target_os = "android")] android_log_dir: &Path,
) -> Result<(), Error> {
    let mut problem_report = ProblemReport::new(redact_custom_strings);

    let daemon_logs_dir = {
        #[cfg(target_os = "android")]
        {
            Ok(android_log_dir.to_owned())
        }
        #[cfg(not(target_os = "android"))]
        {
            mullvad_paths::get_log_dir().map_err(LogError::GetLogDir)
        }
    };

    let daemon_logs = daemon_logs_dir.and_then(list_logs);
    match daemon_logs {
        Ok(daemon_logs) => {
            let mut other_logs = Vec::new();
            for log in daemon_logs {
                match log {
                    Ok(path) => {
                        if is_tunnel_log(&path) {
                            problem_report.add_log(&path);
                        } else {
                            other_logs.push(path);
                        }
                    }
                    Err(error) => problem_report.add_error("Unable to get log path", &error),
                }
            }
            for other_log in other_logs {
                problem_report.add_log(&other_log);
            }
        }
        Err(error) => {
            problem_report.add_error("Failed to list logs in daemon log directory", &error)
        }
    };
    match frontend_log_dir().map(|dir| dir.and_then(list_logs)) {
        Some(Ok(frontend_logs)) => {
            for log in frontend_logs {
                match log {
                    Ok(path) => problem_report.add_log(&path),
                    Err(error) => problem_report.add_error("Unable to get log path", &error),
                }
            }
        }
        Some(Err(error)) => {
            problem_report.add_error("Failed to list logs in frontend log directory", &error)
        }
        None => {}
    }
    #[cfg(target_os = "android")]
    match write_logcat_to_file(android_log_dir) {
        Ok(logcat_path) => problem_report.add_log(&logcat_path),
        Err(error) => problem_report.add_error("Failed to collect logcat", &error),
    }

    problem_report.add_logs(extra_logs);

    write_problem_report(output_path, &problem_report).map_err(|source| Error::WriteReportError {
        path: output_path.display().to_string(),
        source,
    })
}

/// Returns an iterator over all files in the given directory that has the `.log` extension.
fn list_logs(
    log_dir: PathBuf,
) -> Result<impl Iterator<Item = Result<PathBuf, LogError>>, LogError> {
    fs::read_dir(&log_dir)
        .map_err(|source| LogError::ListLogDir {
            path: log_dir.display().to_string(),
            source,
        })
        .map(|dir_entries| {
            let log_extension = Some(OsStr::new("log"));

            dir_entries.filter_map(move |dir_entry_result| match dir_entry_result {
                Ok(dir_entry) => {
                    let path = dir_entry.path();

                    if path.extension() == log_extension {
                        Some(Ok(path))
                    } else {
                        None
                    }
                }
                Err(source) => Some(Err(LogError::ListLogDir {
                    path: log_dir.display().to_string(),
                    source,
                })),
            })
        })
}

/// Returns the directory where the Mullvad GUI frontend stores its logs.
/// If the current platform has a separate directory for frontend logs.
fn frontend_log_dir() -> Option<Result<PathBuf, LogError>> {
    #[cfg(target_os = "linux")]
    {
        Some(
            dirs_next::home_dir()
                .ok_or(LogError::NoHomeDir)
                .map(|home_dir| home_dir.join(".config/Mullvad VPN/logs")),
        )
    }
    #[cfg(target_os = "macos")]
    {
        Some(
            dirs_next::home_dir()
                .ok_or(LogError::NoHomeDir)
                .map(|home_dir| home_dir.join("Library/Logs/Mullvad VPN")),
        )
    }
    #[cfg(target_os = "windows")]
    {
        Some(match std::env::var_os("LOCALAPPDATA") {
            Some(dir) => Ok(Path::new(&dir).join("Mullvad VPN/logs")),
            None => Err(LogError::NoLocalAppDataDir),
        })
    }
    #[cfg(target_os = "android")]
    {
        None
    }
}

fn is_tunnel_log(path: &Path) -> bool {
    match path.file_name() {
        Some(file_name) => file_name.to_string_lossy().contains("openvpn"),
        None => false,
    }
}

#[cfg(target_os = "android")]
fn write_logcat_to_file(log_dir: &Path) -> Result<PathBuf, io::Error> {
    let logcat_path = log_dir.join("logcat.txt");

    duct::cmd!("logcat", "-d")
        .stderr_to_stdout()
        .stdout_path(&logcat_path)
        .run()
        .map(|_| logcat_path)
}

pub fn send_problem_report(
    user_email: &str,
    user_message: &str,
    report_path: &Path,
    cache_dir: &Path,
) -> Result<(), Error> {
    let report_content = normalize_newlines(
        read_file_lossy(report_path, REPORT_MAX_SIZE).map_err(|source| {
            Error::ReadProblemReportError {
                path: report_path.display().to_string(),
                source,
            }
        })?,
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(Error::CreateRuntime)?;
    runtime.block_on(send_problem_report_inner(
        user_email,
        user_message,
        &report_content,
        cache_dir,
    ))
}

async fn send_problem_report_inner(
    user_email: &str,
    user_message: &str,
    report_content: &str,
    cache_dir: &Path,
) -> Result<(), Error> {
    let metadata = ProblemReport::parse_metadata(report_content).unwrap_or_else(metadata::collect);
    let api_runtime = mullvad_api::Runtime::with_cache(
        cache_dir,
        false,
        #[cfg(target_os = "android")]
        None,
    )
    .await
    .map_err(Error::CreateRpcClientError)?;

    let api_client = mullvad_api::ProblemReportProxy::new(
        api_runtime
            .mullvad_rest_handle(
                ApiConnectionMode::try_from_cache(cache_dir)
                    .await
                    .into_repeat(),
                |_| async { true },
            )
            .await,
    );

    for _attempt in 0..MAX_SEND_ATTEMPTS {
        match api_client
            .problem_report(user_email, user_message, report_content, &metadata)
            .await
        {
            Ok(()) => {
                return Ok(());
            }
            Err(error) => {
                if !error.is_network_error() {
                    return Err(Error::SendProblemReportError(error));
                }
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to send problem report due to network error"
                    )
                );
            }
        }
    }
    Err(Error::SendFailedTooManyTimes)
}

fn write_problem_report(path: &Path, problem_report: &ProblemReport) -> io::Result<()> {
    let file = File::create(path)?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)?;
    problem_report.write_to(BufWriter::new(file))?;
    Ok(())
}

#[derive(Debug)]
struct ProblemReport {
    metadata: BTreeMap<String, String>,
    logs: Vec<(String, String)>,
    log_paths: HashSet<PathBuf>,
    redact_custom_strings: Vec<String>,
}

impl ProblemReport {
    /// Creates a new problem report with system information. Logs can be added with `add_log`.
    /// Logs will have all strings in `redact_custom_strings` removed from them.
    pub fn new(mut redact_custom_strings: Vec<String>) -> Self {
        redact_custom_strings.retain(|redact| !redact.is_empty());

        ProblemReport {
            metadata: metadata::collect(),
            logs: Vec::new(),
            log_paths: HashSet::new(),
            redact_custom_strings,
        }
    }

    /// Attach some file logs to this report. This method adds the error chain instead of the log
    /// contents if an error occurs while reading one of the log files.
    pub fn add_logs<I>(&mut self, paths: I)
    where
        I: IntoIterator,
        I::Item: AsRef<Path>,
    {
        for path in paths {
            self.add_log(path.as_ref());
        }
    }

    /// Attach a file log to this report. This method adds the error chain instead of the log
    /// contents if an error occurs while reading the log file.
    pub fn add_log(&mut self, path: &Path) {
        let expanded_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        if self.log_paths.insert(expanded_path.clone()) {
            let redacted_path = self.redact(&expanded_path.to_string_lossy());
            let content = self.redact(&read_file_lossy(path, LOG_MAX_READ_BYTES).unwrap_or_else(
                |error| {
                    error.display_chain_with_msg(&format!(
                        "Error reading the contents of log file: {}",
                        expanded_path.display()
                    ))
                },
            ));
            self.logs.push((redacted_path, content));
            log::info!("Adding {}", expanded_path.display());
        }
    }

    /// Attach an error to the report.
    pub fn add_error(&mut self, message: &'static str, error: &impl ErrorExt) {
        let redacted_error = self.redact(&error.display_chain());
        self.logs.push((message.to_string(), redacted_error));
    }

    fn redact(&self, input: &str) -> String {
        let out1 = Self::redact_account_number(input);
        let out2 = Self::redact_home_dir(&out1);
        let out3 = Self::redact_network_info(&out2);
        let out4 = Self::redact_guids(&out3);
        self.redact_custom_strings(&out4).to_string()
    }

    fn redact_account_number(input: &str) -> Cow<'_, str> {
        lazy_static! {
            static ref RE: Regex = Regex::new("\\d{16}").unwrap();
        }
        RE.replace_all(input, "[REDACTED ACCOUNT NUMBER]")
    }

    fn redact_home_dir(input: &str) -> Cow<'_, str> {
        redact_home_dir_inner(input, dirs_next::home_dir())
    }

    fn redact_network_info(input: &str) -> Cow<'_, str> {
        lazy_static! {
            static ref RE: Regex = {
                let boundary = "[^0-9a-zA-Z.:]";
                let combined_pattern = format!(
                    "(?P<start>^|{})(?:{}|{}|{})",
                    boundary,
                    build_ipv4_regex(),
                    build_ipv6_regex(),
                    build_mac_regex(),
                );
                Regex::new(&combined_pattern).unwrap()
            };
        }
        RE.replace_all(input, "$start[REDACTED]")
    }

    fn redact_guids(input: &str) -> Cow<'_, str> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r#"(?i)\{?[A-F0-9]{8}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{12}\}?"#
            )
            .unwrap();
        }
        RE.replace_all(input, "[REDACTED]")
    }

    fn redact_custom_strings<'a>(&self, input: &'a str) -> Cow<'a, str> {
        // Can probably me made a lot faster with aho-corasick if optimization is ever needed.
        let mut out = Cow::from(input);
        for redact in &self.redact_custom_strings {
            out = out.replace(redact, "[REDACTED]").into()
        }
        out
    }

    fn write_to<W: Write>(&self, mut output: W) -> io::Result<()> {
        // IMPORTANT: Make sure this implementation stays in sync with `parse_metadata` below.
        write_line!(output, "System information:")?;
        for (key, value) in &self.metadata {
            write_line!(output, "{}: {}", key, value)?;
        }
        // Write empty line to separate metadata from first log
        write_line!(output)?;
        for (label, content) in &self.logs {
            write_line!(output, "{}", LOG_DELIMITER)?;
            write_line!(output, "Log: {}", label)?;
            write_line!(output, "{}", LOG_DELIMITER)?;
            output.write_all(content.as_bytes())?;
            write_line!(output)?;
        }
        Ok(())
    }

    /// Tries to parse out the metadata map from a string that is supposed to be a report written by
    /// this struct.
    pub fn parse_metadata(report: &str) -> Option<BTreeMap<String, String>> {
        // IMPORTANT: Make sure this implementation stays in sync with `write_to` above.
        const PATTERN: &str = ": ";
        let mut lines = report.lines();
        if lines.next() != Some("System information:") {
            return None;
        }
        let mut metadata = BTreeMap::new();
        for line in lines {
            // Abort on first empty line, as this is the separator between the metadata and the
            // first log
            if line.is_empty() {
                break;
            }
            let split_i = line.find(PATTERN)?;
            let key = &line[..split_i];
            let value = &line[split_i + PATTERN.len()..];
            metadata.insert(key.to_owned(), value.to_owned());
        }
        Some(metadata)
    }
}

fn redact_home_dir_inner(input: &str, home_dir: Option<PathBuf>) -> Cow<'_, str> {
    match home_dir {
        Some(home) => {
            let out = input.replace(home.to_string_lossy().as_ref(), "~");

            // On Windows, redact the prefix of any path that contains \Users\{user}.
            #[cfg(target_os = "windows")]
            {
                let mut home = home;
                let prefix = home.components().next();
                if let Some(prefix @ std::path::Component::Prefix(_)) = prefix.as_ref() {
                    home = home.strip_prefix(prefix).unwrap().to_path_buf();
                }
                let expr = format!(r"[\w\\]+{}", regex::escape(&home.display().to_string()));
                let regex = Regex::new(&expr).unwrap();

                Cow::Owned(regex.replace_all(&out, "~").to_string())
            }

            #[cfg(not(target_os = "windows"))]
            Cow::from(out)
        }
        None => Cow::from(input),
    }
}

fn build_mac_regex() -> String {
    let octet = "[[:xdigit:]]{2}"; // 0 - ff

    // five pairs of two hexadecimal chars followed by colon or dash
    // followed by a pair of hexadecimal chars
    format!("(?:{octet}[:-]){{5}}({octet})")
}

fn build_ipv4_regex() -> String {
    // regex adapted from  https://www.regular-expressions.info/ip.html

    let above_250 = "25[0-5]";
    let above_200 = "2[0-4][0-9]";
    let above_100 = "1[0-9][0-9]";

    // 100-119 | 120-126 | 128-129 | 130 - 199
    let above_100_not_127 = "1(?:[01][0-9]|2[0-6]|2[89]|[3-9][0-9])";

    let above_0 = "0?[0-9][0-9]?";

    // matches 0-255, except 127
    let first_octet = format!("(?:{above_250}|{above_200}|{above_100_not_127}|{above_0})");

    // matches 0-255
    let ip_octet = format!("(?:{above_250}|{above_200}|{above_100}|{above_0})");

    format!("(?:{first_octet}\\.{ip_octet}\\.{ip_octet}\\.{ip_octet})")
}

fn build_ipv6_regex() -> String {
    // Regular expression obtained from:
    // https://stackoverflow.com/a/17871737
    let ipv4_segment = "(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])";
    let ipv4_address = format!("({ipv4_segment}\\.){{3,3}}{ipv4_segment}");

    let ipv6_segment = "[0-9a-fA-F]{1,4}";

    let long = format!("({ipv6_segment}:){{7,7}}{ipv6_segment}");
    let compressed_1 = format!("({ipv6_segment}:){{1,7}}:");
    let compressed_2 = format!("({ipv6_segment}:){{1,6}}:{ipv6_segment}");
    let compressed_3 = format!("({ipv6_segment}:){{1,5}}(:{ipv6_segment}){{1,2}}");
    let compressed_4 = format!("({ipv6_segment}:){{1,4}}(:{ipv6_segment}){{1,3}}");
    let compressed_5 = format!("({ipv6_segment}:){{1,3}}(:{ipv6_segment}){{1,4}}");
    let compressed_6 = format!("({ipv6_segment}:){{1,2}}(:{ipv6_segment}){{1,5}}");
    let compressed_7 = format!("{ipv6_segment}:((:{ipv6_segment}){{1,6}})");
    let compressed_8 = format!(":((:{ipv6_segment}){{1,7}}|:)");
    let link_local = "[Ff][Ee]80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}";
    let ipv4_mapped = format!("::([fF]{{4}}(:0{{1,4}}){{0,1}}:){{0,1}}{ipv4_address}");
    let ipv4_embedded = format!("({ipv6_segment}:){{1,4}}:{ipv4_address}");

    format!(
        "{long}|{link_local}|{ipv4_mapped}|{ipv4_embedded}|{compressed_8}|{compressed_7}|{compressed_6}|{compressed_5}|{compressed_4}|{compressed_3}|{compressed_2}|{compressed_1}",
    )
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

#[cfg(not(windows))]
fn normalize_newlines(text: String) -> String {
    text
}

#[cfg(windows)]
fn normalize_newlines(text: String) -> String {
    text.replace(LINE_SEPARATOR, "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_ipv4() {
        assert_redacts("1.2.3.4");
        assert_redacts("10.127.0.1");
        assert_redacts("192.168.1.1");
        assert_redacts("10.0.16.1");
        assert_redacts("173.54.12.32");
        assert_redacts("68.4.4.1");
    }

    #[test]
    fn does_not_redact_localhost_ipv4() {
        assert_does_not_redact("127.0.0.1");
    }

    #[test]
    fn redacts_ipv6() {
        assert_redacts("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert_redacts("2001:db8:85a3:0:0:8a2e:370:7334");
        assert_redacts("2001:db8:85a3::8a2e:370:7334");
        assert_redacts("2001:db8:0:0:0:0:2:1");
        assert_redacts("2001:db8::2:1");
        assert_redacts("2001:db8:0000:1:1:1:1:1");
        assert_redacts("2001:db8:0:1:1:1:1:1");
        assert_redacts("2001:db8:0:0:1:0:0:1");
        assert_redacts("2001:db8::1:0:0:1");
        assert_redacts("abcd:dead:beef::");
        assert_redacts("abcd:dead:beef:1234::");
        assert_redacts("::dead:beef:1234");
        assert_redacts("0::0");
        assert_redacts("0:0:0:0::1");
    }

    #[test]
    fn doesnt_redact_not_ipv6() {
        assert_does_not_redact("[talpid_core::firewall]");
    }

    #[test]
    fn redacts_guid() {
        assert_redacts("6B29FC40-CA47-1067-B31D-00DD010662DA");
        assert_redacts("123123ab-12ab-89cd-45ef-012345678901");
        assert_redacts("{123123ab-12ab-89cd-45ef-012345678901}");
    }

    #[test]
    #[cfg(windows)]
    fn redacts_home_dir() {
        let assert_redacts_home_dir = |home_dir, test_str| {
            let input = format!(r"pre {}\remaining\path post", test_str);
            let actual = redact_home_dir_inner(&input, Some(PathBuf::from(home_dir)));
            assert_eq!(r"pre ~\remaining\path post", actual);
        };

        let home_dir = r"C:\Users\user";

        assert_redacts_home_dir(home_dir, r"\Device\HarddiskVolume1\Users\user");
        assert_redacts_home_dir(home_dir, r"C:\Users\user");
    }

    #[test]
    fn doesnt_redact_not_guid() {
        assert_does_not_redact("23123ab-12ab-89cd-45ef-012345678901");
        assert_does_not_redact("GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG");
    }

    #[test]
    fn does_not_redact_time() {
        assert_does_not_redact("09:47:59");
    }

    fn assert_redacts(input: &str) {
        let report = ProblemReport::new(vec![]);
        let actual = report.redact(&format!("pre {input} post"));
        assert_eq!("pre [REDACTED] post", actual);
    }

    fn assert_does_not_redact(input: &str) {
        let report = ProblemReport::new(vec![]);
        let res = report.redact(input);
        assert_eq!(input, res);
    }

    #[test]
    fn parse_metadata() {
        let report = ProblemReport::new(Vec::new());
        let mut report_data = Vec::new();
        report
            .write_to(&mut report_data)
            .expect("Unable to write report to vector");

        let report_string = std::str::from_utf8(&report_data).expect("Report is not correct UTF-8");

        let parsed_metadata = ProblemReport::parse_metadata(report_string)
            .expect("Unable to parse metadata from report");
        let expected_metadata = metadata::collect();

        assert_eq!(parsed_metadata.len(), expected_metadata.len());
        for (key, value) in &expected_metadata {
            let parsed_value = parsed_metadata
                .get(key)
                .expect("Parsed metadata and new one don't match");
            if key == "id" {
                assert_ne!(parsed_value, value, "id not supposed to match");
            } else {
                assert_eq!(parsed_value, value, "value for key '{key}' does not match");
            }
        }
    }
}
