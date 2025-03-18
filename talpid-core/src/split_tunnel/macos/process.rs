//! This module keeps tracks of maintains a list of processes, and keeps it up to date by observing
//! the syscalls `fork`, `exec`, and `exit`.
//! Each process has an exclusion state, based on which paths the process monitor is instructed to
//! exclude.
//! The module currently relies on the `eslogger` tool to do so, which in turn relies on the
//! Endpoint Security framework.

use futures::channel::oneshot;
use libc::pid_t;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};
use talpid_macos::process::{list_pids, process_path};
use talpid_platform_metadata::MacosVersion;
use talpid_types::tunnel::ErrorStateCause;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::OnceCell,
};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
const EARLY_FAIL_TIMEOUT: Duration = Duration::from_millis(500);

static MIN_OS_VERSION: LazyLock<MacosVersion> =
    LazyLock::new(|| MacosVersion::from_raw_version("13.0.0").unwrap());

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Only macOS 13 and later is supported
    #[error("Unsupported macOS version: {actual}, expected at least {}", *MIN_OS_VERSION)]
    UnsupportedMacosVersion {
        actual: talpid_platform_metadata::MacosVersion,
    },
    /// Failed to start eslogger listener
    #[error("Failed to start eslogger")]
    StartMonitor(#[source] io::Error),
    /// The app requires TCC approval from the user.
    #[error("The app needs TCC approval from the user for Full Disk Access")]
    NeedFullDiskPermissions,
    /// eslogger failed due to an unknown error
    #[error("eslogger returned an error")]
    MonitorFailed(#[source] io::Error),
    /// Monitor task panicked
    #[error("Monitor task panicked")]
    MonitorTaskPanicked(#[source] tokio::task::JoinError),
    /// Failed to list processes
    #[error("Failed to list processes")]
    InitializePids(#[source] io::Error),
    /// Failed to find path for a process
    #[error("Failed to find path for a process: {}", _0)]
    FindProcessPath(#[source] io::Error, pid_t),
}

impl From<&Error> for ErrorStateCause {
    fn from(value: &Error) -> Self {
        match value {
            Error::NeedFullDiskPermissions => ErrorStateCause::NeedFullDiskPermissions,
            _ => ErrorStateCause::SplitTunnelError,
        }
    }
}

pub struct ProcessMonitor(());

#[derive(Debug)]
pub struct ProcessMonitorHandle {
    stop_proc_tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
    proc_task: tokio::task::JoinHandle<Result<(), Error>>,
    states: ProcessStates,
}

impl ProcessMonitor {
    pub async fn spawn() -> Result<ProcessMonitorHandle, Error> {
        check_os_version_support()?;

        if !has_full_disk_access().await {
            return Err(Error::NeedFullDiskPermissions);
        }

        let states = ProcessStates::new()?;
        let proc = spawn_eslogger()?;
        let (stop_proc_tx, stop_rx): (_, oneshot::Receiver<oneshot::Sender<_>>) =
            oneshot::channel();
        let proc_task = tokio::spawn(handle_eslogger_output(proc, states.clone(), stop_rx));

        Ok(ProcessMonitorHandle {
            stop_proc_tx: Some(stop_proc_tx),
            proc_task,
            states,
        })
    }
}

/// Return whether the process has full-disk access
/// If it cannot be determined that access is available, it is assumed to be available
pub async fn has_full_disk_access() -> bool {
    static HAS_TCC_APPROVAL: OnceCell<bool> = OnceCell::const_new();
    *HAS_TCC_APPROVAL
        .get_or_try_init(|| async {
            let mut proc = spawn_eslogger()?;

            let stdout = proc.stdout.take().unwrap();
            let stderr = proc.stderr.take().unwrap();
            drop(proc.stdin.take());

            let has_full_disk_access = parse_logger_status(stdout, stderr).await == NeedFda::No;
            Ok::<bool, Error>(has_full_disk_access)
        })
        .await
        .unwrap_or(&true)
}

#[derive(Debug, PartialEq)]
enum NeedFda {
    Yes,
    No,
}

/// Return whether `proc` reports that full-disk access is unavailable based on its output
/// If it cannot be determined that access is available, it is assumed to be available
async fn parse_logger_status(
    stdout: impl AsyncRead + Unpin + Send + 'static,
    stderr: impl AsyncRead + Unpin + Send + 'static,
) -> NeedFda {
    let stderr = BufReader::new(stderr);
    let mut stderr_lines = stderr.lines();

    let stdout = BufReader::new(stdout);
    let mut stdout_lines = stdout.lines();

    let mut need_full_disk_access = tokio::spawn(async move {
        tokio::select! {
            biased; result = stderr_lines.next_line() => {
                let Ok(Some(line)) = result else {
                    return NeedFda::No;
                };
                if let Some(Error::NeedFullDiskPermissions) = parse_eslogger_error(&line) {
                    return NeedFda::Yes;
                }
                NeedFda::No
            }
            Ok(Some(_)) = stdout_lines.next_line() => {
                // Received output, but not an err
                NeedFda::No
            }
        }
    });

    let deadline = tokio::time::sleep(EARLY_FAIL_TIMEOUT);

    tokio::select! {
        // Received standard err/out
        biased; need_full_disk_access = &mut need_full_disk_access => {
            need_full_disk_access.unwrap_or(NeedFda::No)
        }
        // Timed out while checking for full-disk access
        _ = deadline => NeedFda::No,
    }
}

/// Run until the process exits or `stop_rx` is signaled
async fn handle_eslogger_output(
    mut proc: tokio::process::Child,
    states: ProcessStates,
    stop_rx: oneshot::Receiver<oneshot::Sender<()>>,
) -> Result<(), Error> {
    let stdout = proc.stdout.take().unwrap();
    let stderr = proc.stderr.take().unwrap();

    // Parse each line from stdout as an ESMessage
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            // Each line from eslogger is a JSON object, one of several types of messages;
            // see `ESMessage`
            let val: ESMessage = match serde_json::from_str(&line) {
                Ok(val) => val,
                Err(error) => {
                    log::error!("Failed to parse eslogger message: {error}");
                    continue;
                }
            };

            let mut inner = states.inner.lock().unwrap();
            inner.handle_message(val);
        }
    });

    // Store the most recent stderr line in case we need to return an error
    let last_stderr = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        let mut last_error = None;

        while let Ok(Some(line)) = lines.next_line().await {
            last_error = Some(line);
        }
        last_error
    });

    // Wait for a stop signal or process exit
    let result = tokio::select! {
        result = proc.wait() => {
            match result {
                Ok(status) => {
                    if let Ok(Some(last_error)) = last_stderr.await {
                        log::error!("eslogger error: {last_error}");
                        if let Some(error) = parse_eslogger_error(&last_error) {
                            return Err(error);
                        }
                    }
                    Err(Error::MonitorFailed(io::Error::other(format!("eslogger stopped unexpectedly: {status}"))))
                }
                Err(error) => Err(Error::MonitorFailed(error)),
            }
        }
        Ok(response_tx) = stop_rx => {
            if let Err(error) = proc.kill().await {
                log::error!("Failed to kill eslogger: {error}");
            }
            if tokio::time::timeout(SHUTDOWN_TIMEOUT, proc.wait())
                .await
                .is_err()
            {
                log::error!("Failed to wait for ST process handler");
            }
            let _ = response_tx.send(());

            Ok(())
        }
    };

    log::debug!("Process monitor stopped");

    result
}

/// Launch a new instance of `eslogger`, listening for exec, fork, and exit syscalls
fn spawn_eslogger() -> Result<tokio::process::Child, Error> {
    let mut cmd = tokio::process::Command::new("/usr/bin/eslogger");
    cmd.args(["exec", "fork", "exit"])
        .kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn().map_err(Error::StartMonitor)
}

impl ProcessMonitorHandle {
    pub async fn shutdown(&mut self) {
        let Some(stop_tx) = self.stop_proc_tx.take() else {
            return;
        };

        let (tx, rx) = oneshot::channel();
        let _ = stop_tx.send(tx);
        let _ = rx.await;
    }

    pub async fn wait(&mut self) -> Result<(), Error> {
        (&mut self.proc_task)
            .await
            .map_err(Error::MonitorTaskPanicked)?
    }

    pub fn states(&self) -> &ProcessStates {
        &self.states
    }
}

/// Controls the known exclusion states of all processes
#[derive(Debug, Clone)]
pub struct ProcessStates {
    inner: Arc<Mutex<InnerProcessStates>>,
}

/// Possible states of each process
#[derive(Debug, Clone)]
pub enum ExclusionStatus {
    /// The process should be excluded from the VPN
    Excluded,
    /// The process should not be excluded from the VPN
    Included,
    /// The process is unknown
    Unknown,
}

#[derive(Debug)]
struct InnerProcessStates {
    processes: HashMap<pid_t, ProcessInfo>,
    exclude_paths: HashSet<PathBuf>,
}

impl ProcessStates {
    /// Initialize process states
    fn new() -> Result<Self, Error> {
        let mut states = InnerProcessStates {
            processes: HashMap::new(),
            exclude_paths: HashSet::new(),
        };

        let processes = list_pids().map_err(Error::InitializePids)?;

        for pid in processes {
            let path = process_path(pid).map_err(|error| Error::FindProcessPath(error, pid))?;
            states.processes.insert(pid, ProcessInfo::included(path));
        }

        Ok(ProcessStates {
            inner: Arc::new(Mutex::new(states)),
        })
    }

    pub fn exclude_paths(&self, paths: HashSet<PathBuf>) {
        let mut inner = self.inner.lock().unwrap();

        for info in inner.processes.values_mut() {
            // Remove no-longer excluded paths from exclusion list
            let mut new_exclude_paths: HashSet<_> = info
                .excluded_by_paths
                .intersection(&paths)
                .cloned()
                .collect();

            // Check if own path is excluded
            if paths.contains(&info.exec_path) && !new_exclude_paths.contains(&info.exec_path) {
                new_exclude_paths.insert(info.exec_path.clone());
            }

            info.excluded_by_paths = new_exclude_paths;
        }

        inner.exclude_paths = paths;
    }

    pub fn get_process_status(&self, pid: pid_t) -> ExclusionStatus {
        let inner = self.inner.lock().unwrap();
        match inner.processes.get(&pid) {
            Some(val) if val.is_excluded() => ExclusionStatus::Excluded,
            Some(_) => ExclusionStatus::Included,
            None => ExclusionStatus::Unknown,
        }
    }
}

impl InnerProcessStates {
    fn handle_message(&mut self, msg: ESMessage) {
        let pid = msg.process.audit_token.pid;

        match msg.event {
            ESEvent::Fork(evt) => self.handle_fork(pid, msg.process.executable.path, evt),
            ESEvent::Exec(evt) => self.handle_exec(pid, evt),
            ESEvent::Exit {} => self.handle_exit(pid),
        }
    }

    // For new processes, inherit all exclusion state from the parent, if there is one.
    // Otherwise, look up excluded paths
    fn handle_fork(&mut self, parent_pid: pid_t, exec_path: PathBuf, msg: ESForkEvent) {
        let pid = msg.child.audit_token.pid;

        if self.processes.contains_key(&pid) {
            log::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent
        let base_info = match self.processes.get(&parent_pid) {
            Some(parent_info) => parent_info.to_owned(),
            None => {
                log::error!("{pid}: Unknown parent pid {parent_pid}!");
                ProcessInfo::included(exec_path)
            }
        };

        // no exec yet; only pid and parent pid change
        if base_info.is_excluded() {
            log::trace!(
                "{pid} excluded (inherited from {parent_pid}) (exclude paths: {:?}",
                base_info.excluded_by_paths
            );
        }

        self.processes.insert(pid, base_info);
    }

    fn handle_exec(&mut self, pid: pid_t, msg: ESExecEvent) {
        let Some(info) = self.processes.get_mut(&pid) else {
            log::error!("exec received for unknown pid {pid}");
            return;
        };
        if msg.target.executable.path_truncated {
            log::error!(
                "Ignoring process {pid} with truncated path: {}",
                msg.target.executable.path
            );
            return;
        }

        info.exec_path = PathBuf::from(msg.target.executable.path);

        // If the path is already excluded, no need to add it again
        if info.excluded_by_paths.contains(&info.exec_path) {
            return;
        }

        // Exclude if path is excluded
        if self.exclude_paths.contains(&info.exec_path) {
            info.excluded_by_paths.insert(info.exec_path.clone());
            log::trace!("Excluding {pid} by path: {}", info.exec_path.display());
        }
    }

    fn handle_exit(&mut self, pid: pid_t) {
        if self.processes.remove(&pid).is_none() {
            log::error!("exit syscall for unknown pid {pid}");
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    exec_path: PathBuf,
    excluded_by_paths: HashSet<PathBuf>,
}

impl ProcessInfo {
    fn included(exec_path: PathBuf) -> Self {
        ProcessInfo {
            exec_path,
            excluded_by_paths: HashSet::new(),
        }
    }

    fn is_excluded(&self) -> bool {
        !self.excluded_by_paths.is_empty()
    }
}

/// `fork` event details
#[derive(Debug, Deserialize)]
struct ESForkChild {
    audit_token: ESAuditToken,
}

/// `fork` event returned by `eslogger`
#[derive(Debug, Deserialize)]
struct ESForkEvent {
    child: ESForkChild,
}

/// `exec` event returned by `eslogger`
#[derive(Debug, Deserialize)]
struct ESExecEvent {
    target: EsExecTarget,
}

/// `target` field of the `exec` event returned by `eslogger`
/// See `eslogger exec` output.
#[derive(Debug, Deserialize)]
struct EsExecTarget {
    executable: EsExecTargetExecutable,
}

/// `executable` field of the `exec` `target` information
/// See `eslogger exec` output.
#[derive(Debug, Deserialize)]
struct EsExecTargetExecutable {
    path: String,
    path_truncated: bool,
}

/// Event that triggered the message returned by `eslogger`.
/// See the `es_events_t` struct for more information:
/// https://developer.apple.com/documentation/endpointsecurity/es_message_t/3228969-event?language=objc
/// A list of all event types can be found here:
/// https://developer.apple.com/documentation/endpointsecurity/es_event_type_t/es_event_type_notify_fork?language=objc
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ESEvent {
    Fork(ESForkEvent),
    Exec(ESExecEvent),
    Exit {},
}

/// Message containing the path to the image of the process.
/// This message is analogous to the `executable` field of `es_process_t`:
/// https://developer.apple.com/documentation/endpointsecurity/es_process_t/3228975-audit_token?language=objc
#[derive(Debug, Deserialize)]
struct ESExecutable {
    path: PathBuf,
}

/// Message containing the process identifier of the process.
/// This message is analogous to the `audit_token` field of `es_process_t`:
/// https://developer.apple.com/documentation/endpointsecurity/es_process_t/3228975-audit_token?language=objc
#[derive(Debug, Deserialize)]
struct ESAuditToken {
    pid: pid_t,
}

/// Process information for the message returned by `eslogger`.
/// This message is analogous to the `es_process_t` struct:
/// https://developer.apple.com/documentation/endpointsecurity/es_process_t?language=objc
#[derive(Debug, Deserialize)]
struct ESProcess {
    audit_token: ESAuditToken,
    executable: ESExecutable,
}

/// This struct represents each message returned by eslogger
/// This message is analogous to the `es_message_t` struct:
/// https://developer.apple.com/documentation/endpointsecurity/es_message_t?language=objc
#[derive(Debug, Deserialize)]
struct ESMessage {
    event: ESEvent,
    process: ESProcess,
}

fn parse_eslogger_error(stderr_str: &str) -> Option<Error> {
    if stderr_str.contains("ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED") {
        Some(Error::NeedFullDiskPermissions)
    } else {
        None
    }
}

/// Check whether the current macOS version is supported, and return an error otherwise
fn check_os_version_support() -> Result<(), Error> {
    match MacosVersion::new() {
        Ok(version) => check_os_version_support_inner(version),
        Err(error) => {
            log::error!("Failed to detect macOS version: {error}");
            Ok(())
        }
    }
}

fn check_os_version_support_inner(version: MacosVersion) -> Result<(), Error> {
    if version >= *MIN_OS_VERSION {
        Ok(())
    } else {
        Err(Error::UnsupportedMacosVersion { actual: version })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::time::Duration;
    use talpid_platform_metadata::MacosVersion;
    use tokio::io::{simplex, AsyncWriteExt};

    /// A mock-version of std{out,err}. [tokio::io::SimplexStream] implements [AsyncRead], so it can be used to test
    /// [parse_logger_status].
    fn output(msg: &'static str, lag: Duration) -> impl AsyncRead + Unpin + Send + 'static {
        // Ensure that 'msg' contains a newline to prevent user errors
        assert!(
            msg.contains('\n'),
            "Message does not contain a newline!! Make sure to add a newline to '{msg}'"
        );
        let (stdout_read, mut stdout_write) = simplex(msg.len());
        //  "print" to "stdout" after `duration`.
        tokio::spawn(async move {
            tokio::time::sleep(lag).await;
            stdout_write.write_all(msg.as_bytes()).await.unwrap();
        });
        stdout_read
    }

    #[test]
    fn test_min_os_version() {
        assert!(check_os_version_support_inner(MIN_OS_VERSION.clone()).is_ok());

        // test unsupported version
        assert!(
            check_os_version_support_inner(MacosVersion::from_raw_version("12.1").unwrap())
                .is_err()
        );

        // test supported version
        assert!(
            check_os_version_support_inner(MacosVersion::from_raw_version("13.0").unwrap()).is_ok()
        );
    }

    /// If the process prints 'ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED' to stderr, full-disk access
    /// is denied.
    #[tokio::test]
    async fn test_parse_logger_status_missing_access() {
        let need_fda = parse_logger_status(
            &[][..],
            b"ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED\n".as_slice(),
        )
        .await;

        assert_eq!(
            need_fda,
            NeedFda::Yes,
            "expected 'NeedFda::Yes' when ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED was present"
        );
    }

    /// If process exits without 'ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED', assume full-disk access
    /// is available.
    #[tokio::test]
    async fn test_parse_logger_status_immediate_exit() {
        let need_fda = parse_logger_status(
            b"nothing to see here\n".as_slice(),
            b"nothing to see here\n".as_slice(),
        )
        .await;

        assert_eq!(
            need_fda,
            NeedFda::No,
            "expected 'NeedFda::No' on immediate exit",
        );
    }

    /// Check that [parse_logger_status] returns within a reasonable timeframe.
    /// "Reasonable" being within [EARLY_FAIL_TIMEOUT].
    #[tokio::test(start_paused = true)]
    async fn test_parse_logger_status_responsive() {
        let start = tokio::time::Instant::now();
        let stdout = output("This will never be printed\n", Duration::MAX);
        let stderr = output(
            "ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED\n",
            EARLY_FAIL_TIMEOUT / 2,
        );
        tokio::time::resume();

        let need_fda = parse_logger_status(stdout, stderr).await;

        tokio::time::pause();

        assert_eq!(
            need_fda,
            NeedFda::Yes,
            "expected 'NeedFda::Yes' when ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED was eventually printed to stderr"
        );

        // Assert that we did not spend more time waiting than we should
        assert!(start.elapsed() < EARLY_FAIL_TIMEOUT);
    }

    /// Check that [parse_logger_status] doesn't get stuck because nothing is ever output
    /// to std{out,err}. It should time out with the assumption that full-disk access is available.
    #[tokio::test]
    async fn test_parse_logger_status_timeout() {
        let stdout = output("This will never be printed\n", Duration::MAX);
        let stderr = output("This will never be printed\n", Duration::MAX);

        let need_fda = parse_logger_status(stdout, stderr).await;

        assert_eq!(
            need_fda,
            NeedFda::No,
            "expected 'NeedFda::No' when nothing was ever printed to stdout or stderr"
        );
    }
}
