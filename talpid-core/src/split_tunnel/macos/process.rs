//! This module keeps tracks of maintains a list of processes, and keeps it up to date by observing
//! the syscalls `fork`, `exec`, and `exit`.
//! Each process has an exclusion state, based on which paths the process monitor is instructed to
//! exclude.
//! The module currently relies on the `eslogger` tool to do so, which in turn relies on the
//! Endpoint Security framework.

use either::Either;
use futures::channel::oneshot;
use libc::pid_t;
use serde::{Deserialize, de::Error as _};
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
        let Some(pid) = msg.process.audit_token.checked_pid() else {
            log::trace!("eslogger returned bad pid: {msg:?}");
            return;
        };

        match msg.event {
            ESEvent::Fork(evt) => self.handle_fork(pid, msg.process.executable.path, evt),
            ESEvent::Exec(evt) => self.handle_exec(pid, evt),
            ESEvent::Exit {} => self.handle_exit(pid),
        }
    }

    // For new processes, inherit all exclusion state from the parent, if there is one.
    // Otherwise, look up excluded paths
    fn handle_fork(&mut self, parent_pid: pid_t, exec_path: PathBuf, msg: ESForkEvent) {
        let Some(pid) = msg.child.audit_token.checked_pid() else {
            log::trace!("eslogger returned bad pid: {msg:?}");
            return;
        };

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
#[derive(Debug)]
struct ESAuditToken {
    pid: pid_t,
}

/// Custom [Deserialize] impl for [ESAuditToken] because they changed the representation of it in
/// version 10 of the JSON schema.
///
/// # Version 9
/// JSON object. Self-explanatory.
/// ```json
/// "audit_token":{
///   "egid":0,
///   "pid":12072,
///   "ruid":0,
///   "asid":100017,
///   "euid":0,
///   "pidversion":172341,
///   "auid":4294967295,
///   "rgid":0
/// }
/// ```
///
/// # Version 10
/// A list, where the fields are stored in a certain order.
/// ```json
/// "audit_token": [
///   501,    // probably auid?
///   501,    // probably euid?
///   20,     // probably egid?
///   501,    // probably ruid?
///   20,     // probably rgid?
///   21497,  // pid
///   100013, // probably asid?
///   38282   // probably pidversion?
/// ]
/// ```
impl<'de> Deserialize<'de> for ESAuditToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as i64s because not all fields fit into a pid_t
        let value: Either<Vec<i64>, HashMap<&str, i64>> =
            either::serde_untagged::deserialize(deserializer)?;

        let pid = match value {
            Either::Left(list) => match list[..] {
                [_auid, _euid, _egid, _ruid, _rgid, pid, _asid, _] => pid,
                _ => return Err(D::Error::custom("Expected list with exactly 8 elements")),
            },

            Either::Right(mut object) => object
                .remove("pid")
                .ok_or_else(|| D::Error::custom("Missing field 'pid'"))?,
        };

        let pid = pid_t::try_from(pid).map_err(|e| D::Error::custom(e.to_string()))?;

        Ok(ESAuditToken { pid })
    }
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
    #[allow(dead_code)]
    version: SupportedVersion,
    event: ESEvent,
    process: ESProcess,
}

/// An `i32`-wrapper that verifies that the [ESMessage] version is supported.
#[derive(Debug)]
#[allow(dead_code)]
struct SupportedVersion(i32);

impl<'de> Deserialize<'de> for SupportedVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version = i32::deserialize(deserializer)?;

        match version {
            0..=10 => Ok(SupportedVersion(version)),

            // We don't know how to deserialize anything past version 10
            _ => Err(D::Error::custom(format!(
                "Unsupported ESMessage version: {version}"
            ))),
        }
    }
}

impl ESAuditToken {
    /// Check that `pid` is positive and return it.
    pub fn checked_pid(&self) -> Option<pid_t> {
        (self.pid > 0).then_some(self.pid)
    }
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
    use tokio::io::{AsyncWriteExt, simplex};

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

    /// Assert that we can deserialize output from different versions of `eslogger` into valid
    /// [ESMessage]s.
    #[test]
    fn test_deserialize_esmessage() {
        let valid_esmessages = [
            // version 9, taken from macOS 15
            r#"{"process":{"codesigning_flags":637623057,"cdhash":"F988105881118CD77EF87293D97DECE8E193FA98","session_id":532,"ppid":532,"group_id":532,"is_platform_binary":true,"team_id":null,"audit_token":{"euid":0,"rgid":0,"egid":0,"pid":11221,"asid":100017,"ruid":0,"auid":4294967295,"pidversion":170692},"responsible_audit_token":{"pid":532,"asid":100017,"rgid":0,"auid":4294967295,"euid":0,"ruid":0,"pidversion":1294,"egid":0},"is_es_client":false,"signing_id":"com.apple.ipconfig","start_time":"2025-08-07T15:13:09.798115Z","original_ppid":532,"parent_audit_token":{"pid":532,"asid":100017,"rgid":0,"auid":4294967295,"euid":0,"ruid":0,"pidversion":1294,"egid":0},"executable":{"path_truncated":false,"stat":{"st_gid":0,"st_ino":1152921500312525701,"st_ctimespec":"2025-07-09T06:27:14.000000000Z","st_mtimespec":"2025-07-09T06:27:14.000000000Z","st_gen":0,"st_atimespec":"2025-07-09T06:27:14.000000000Z","st_dev":16777234,"st_uid":0,"st_rdev":0,"st_birthtimespec":"2025-07-09T06:27:14.000000000Z","st_mode":33261,"st_nlink":1,"st_size":259504,"st_blocks":152,"st_flags":524320,"st_blksize":4096},"path":"\/usr\/sbin\/ipconfig"},"tty":null},"time":"2025-08-07T15:13:09.811587464Z","seq_num":362,"action":{"result":{"result_type":0,"result":{"auth":0}}},"event_type":15,"event":{"exit":{"stat":0}},"mach_time":401374797834,"version":9,"thread":{"thread_id":622243},"global_seq_num":1103,"schema_version":1,"action_type":1}"#,
            // version 9, taken from macOS 15
            r#"{"action":{"result":{"result":{"auth":0},"result_type":0}},"event_type":9,"global_seq_num":75,"action_type":1,"mach_time":289350913517,"process":{"is_platform_binary":false,"team_id":null,"signing_id":"nu-b9fb5b9dbba2e494","cdhash":"28CD2C759132B07D63C3A2B377AD440A6C66098E","executable":{"stat":{"st_size":38146864,"st_gid":80,"st_ino":13982224,"st_uid":501,"st_ctimespec":"2025-05-09T11:57:32.842789602Z","st_gen":0,"st_mtimespec":"2025-04-29T23:31:45.000000000Z","st_blocks":74512,"st_rdev":0,"st_dev":16777234,"st_atimespec":"2025-08-07T12:32:35.400606076Z","st_nlink":1,"st_mode":33133,"st_blksize":4096,"st_birthtimespec":"2025-04-29T23:31:45.000000000Z","st_flags":0},"path_truncated":false,"path":"\/bin\/nu"},"group_id":97391,"parent_audit_token":{"asid":100019,"ruid":501,"pidversion":87724,"egid":20,"rgid":20,"pid":58916,"euid":501,"auid":501},"session_id":58916,"audit_token":{"auid":501,"asid":100019,"egid":20,"pid":97391,"pidversion":149091,"rgid":20,"euid":501,"ruid":501},"ppid":58916,"responsible_audit_token":{"asid":100019,"ruid":501,"pidversion":2424,"egid":20,"rgid":20,"pid":938,"euid":501,"auid":501},"original_ppid":58916,"codesigning_flags":570556931,"start_time":"2025-08-07T13:21:35.877422Z","tty":{"stat":{"st_size":0,"st_gid":4,"st_uid":501,"st_ino":1223,"st_ctimespec":"2025-08-07T13:21:35.878404000Z","st_gen":0,"st_mtimespec":"2025-08-07T13:21:35.878404000Z","st_blocks":0,"st_rdev":268435459,"st_dev":1333267060,"st_atimespec":"2025-08-07T13:21:35.874434000Z","st_mode":8592,"st_nlink":1,"st_blksize":65536,"st_birthtimespec":"1970-01-01T00:00:00.000000000Z","st_flags":0},"path_truncated":false,"path":"\/dev\/ttys003"},"is_es_client":false},"time":"2025-08-07T13:21:35.880814738Z","seq_num":27,"version":9,"event":{"exec":{"cwd":{"stat":{"st_size":2624,"st_gid":20,"st_uid":501,"st_ino":539935,"st_ctimespec":"2025-08-07T12:36:24.368103159Z","st_gen":0,"st_mtimespec":"2025-08-07T12:36:24.368103159Z","st_blocks":0,"st_rdev":0,"st_dev":16777234,"st_atimespec":"2025-08-07T12:36:24.414321469Z","st_mode":16877,"st_nlink":82,"st_blksize":4096,"st_birthtimespec":"2024-09-25T14:04:33.178667447Z","st_flags":0},"path_truncated":false,"path":"\/bin\/mullvadvpn-app"},"env":["FOO=bar"],"target":{"team_id":null,"is_platform_binary":false,"signing_id":"connection_checker-04fde7bdb8bceee3","cdhash":"33F0A3D85BEA260FED5CAD0529AB0E84EC9A0DF1","executable":{"stat":{"st_size":5087360,"st_gid":20,"st_ino":14784254,"st_uid":501,"st_ctimespec":"2025-05-12T14:47:04.129389008Z","st_gen":0,"st_mtimespec":"2025-05-12T14:47:04.107254919Z","st_blocks":9944,"st_rdev":0,"st_dev":16777234,"st_atimespec":"2025-08-07T13:21:35.888621297Z","st_nlink":1,"st_mode":33261,"st_blksize":4096,"st_birthtimespec":"2025-05-12T14:47:04.106815000Z","st_flags":0},"path_truncated":false,"path":"\/bin\/connection-checker"},"group_id":97391,"parent_audit_token":{"pidversion":87724,"rgid":20,"pid":58916,"egid":20,"ruid":501,"euid":501,"asid":100019,"auid":501},"session_id":58916,"audit_token":{"pid":97391,"rgid":20,"euid":501,"auid":501,"egid":20,"asid":100019,"pidversion":149092,"ruid":501},"ppid":58916,"responsible_audit_token":{"pidversion":2424,"rgid":20,"pid":938,"egid":20,"ruid":501,"euid":501,"asid":100019,"auid":501},"original_ppid":58916,"codesigning_flags":570556931,"start_time":"2025-08-07T13:21:35.877422Z","tty":{"stat":{"st_size":0,"st_gid":4,"st_ino":1223,"st_uid":501,"st_ctimespec":"2025-08-07T13:21:35.878404000Z","st_gen":0,"st_mtimespec":"2025-08-07T13:21:35.878404000Z","st_blocks":0,"st_rdev":268435459,"st_dev":1333267060,"st_mode":8592,"st_nlink":1,"st_atimespec":"2025-08-07T13:21:35.874434000Z","st_blksize":65536,"st_birthtimespec":"1970-01-01T00:00:00.000000000Z","st_flags":0},"path_truncated":false,"path":"\/dev\/ttys003"},"is_es_client":false},"last_fd":9,"image_cpusubtype":0,"fds":[{"fdtype":1,"fd":0},{"fdtype":1,"fd":1},{"fdtype":1,"fd":2},{"fdtype":1,"fd":5},{"fdtype":1,"fd":6},{"fdtype":1,"fd":8},{"fdtype":1,"fd":9}],"image_cputype":16777228,"args":["\/bin\/connection-checker"],"dyld_exec_path":"\/bin\/connection-checker","script":null}},"thread":{"thread_id":505819},"schema_version":1}"#,
            // version 10, taken from macOS 26
            r#"{"version":10,"event":{"fork":{"child":{"signing_id":"net.mullvad.vpn","audit_token":[501,501,20,501,20,21497,100013,38282],"ppid":19745,"team_id":"CKG9MXH72F","parent_audit_token":[501,501,20,501,20,19745,100013,35165],"session_id":1,"group_id":19745,"cs_validation_category":6,"responsible_audit_token":[501,501,20,501,20,19745,100013,35165],"is_platform_binary":false,"tty":null,"is_es_client":false,"original_ppid":19745,"executable":{"path":"\/Applications\/Mullvad VPN.app\/Contents\/MacOS\/Mullvad VPN","stat":{"st_ctimespec":"2025-07-22T15:04:55.459801307Z","st_dev":16777234,"st_gid":0,"st_atimespec":"2025-07-22T15:05:14.889095406Z","st_blocks":272,"st_blksize":4096,"st_rdev":0,"st_mode":33261,"st_birthtimespec":"2025-06-23T14:48:03.000000000Z","st_size":135216,"st_flags":0,"st_uid":0,"st_mtimespec":"2025-06-23T14:48:03.000000000Z","st_ino":78460340,"st_nlink":1,"st_gen":0},"path_truncated":false},"codesigning_flags":570491649,"start_time":"2025-07-22T15:09:01.083979Z","cdhash":"C26BC5CF81E08B87DF707685A8EA3652446977F1"}}},"thread":{"thread_id":227846},"time":"2025-07-22T15:09:01.084030274Z","seq_num":0,"schema_version":1,"event_type":11,"action":{"result":{"result":{"auth":0},"result_type":0}},"global_seq_num":1,"process":{"team_id":"CKG9MXH72F","original_ppid":1,"audit_token":[501,501,20,501,20,19745,100013,35165],"signing_id":"net.mullvad.vpn","start_time":"2025-07-22T15:05:14.076236Z","responsible_audit_token":[501,501,20,501,20,19745,100013,35165],"parent_audit_token":[4294967295,0,0,0,0,1,100012,1029],"ppid":1,"codesigning_flags":570491649,"tty":null,"is_es_client":false,"group_id":19745,"session_id":1,"cs_validation_category":6,"executable":{"path_truncated":false,"path":"\/Applications\/Mullvad VPN.app\/Contents\/MacOS\/Mullvad VPN","stat":{"st_size":135216,"st_atimespec":"2025-07-22T15:05:14.889095406Z","st_mode":33261,"st_blocks":272,"st_ctimespec":"2025-07-22T15:04:55.459801307Z","st_uid":0,"st_gen":0,"st_blksize":4096,"st_gid":0,"st_rdev":0,"st_birthtimespec":"2025-06-23T14:48:03.000000000Z","st_nlink":1,"st_dev":16777234,"st_flags":0,"st_ino":78460340,"st_mtimespec":"2025-06-23T14:48:03.000000000Z"}},"is_platform_binary":false,"cdhash":"C26BC5CF81E08B87DF707685A8EA3652446977F1"},"action_type":1,"mach_time":241051246374}"#,
            // version 11 doesn't exist at the time of writing
            r#"{"version":11}"#,
        ];

        for s in valid_esmessages {
            let result = serde_json::from_str::<ESMessage>(s);
            insta::assert_debug_snapshot!((s, result));
        }
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
