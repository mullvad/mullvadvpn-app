//! This module keeps tracks of maintains a list of processes, and keeps it up to date by observing
//! the syscalls `fork`, `exec`, and `exit`.
//! Each process has an exclusion state, based on which paths the process monitor is instructed to
//! exclude.
//! The module currently relies on the `eslogger` tool to do so, which in turn relies on the
//! Endpoint Security framework.

use futures::channel::oneshot;
use libc::{proc_listallpids, proc_pidpath};
use serde::Deserialize;
use std::collections::HashSet;
use std::{
    collections::HashMap,
    ffi::c_void,
    io,
    path::PathBuf,
    process::Stdio,
    ptr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, BufReader};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
const EARLY_FAIL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to start eslogger listener
    #[error("Failed to start eslogger")]
    StartMonitor(#[source] io::Error),
    /// eslogger failed
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
    FindProcessPath(#[source] io::Error, u32),
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
        let states = ProcessStates::new()?;

        let mut cmd = tokio::process::Command::new("/usr/bin/eslogger");
        cmd.args(["exec", "fork", "exit"])
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut proc = cmd.spawn().map_err(Error::StartMonitor)?;

        let stdout = proc.stdout.take().unwrap();

        let states_clone = states.clone();

        let (stop_proc_tx, stop_rx): (_, oneshot::Receiver<oneshot::Sender<_>>) =
            oneshot::channel();

        let mut proc_task = tokio::spawn(async move {
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

                    let mut inner = states_clone.inner.lock().unwrap();
                    inner.handle_message(val);
                }
            });

            let result = tokio::select! {
                result = proc.wait() => {
                    match result {
                        Ok(status) => {
                            Err(Error::MonitorFailed(io::Error::new(io::ErrorKind::Other, format!("eslogger stopped unexpectedly: {status}"))))
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
        });

        match tokio::time::timeout(EARLY_FAIL_TIMEOUT, &mut proc_task).await {
            // On timeout, all is well
            Err(_) => (),
            // The process returned an error
            Ok(Ok(Err(error))) => return Err(error),
            Ok(Ok(Ok(()))) => unreachable!("process monitor stopped prematurely"),
            Ok(Err(_)) => unreachable!("process monitor panicked"),
        }

        Ok(ProcessMonitorHandle {
            stop_proc_tx: Some(stop_proc_tx),
            proc_task,
            states,
        })
    }
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
    processes: HashMap<u32, ProcessInfo>,
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

        for (_pid, info) in &mut inner.processes {
            // Remove no-longer excluded paths from exclusion list
            let mut new_exclude_paths: HashSet<_> = info
                .excluded_by_paths
                .intersection(&paths)
                .cloned()
                .collect();

            // Check if own path is excluded
            if paths.contains(&info.exec_path) && !new_exclude_paths.contains(&info.exec_path) {
                new_exclude_paths.insert(info.exec_path.to_owned());
            }

            info.excluded_by_paths = new_exclude_paths;
        }

        inner.exclude_paths = paths;
    }

    pub fn get_process_status(&self, pid: u32) -> ExclusionStatus {
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
    fn handle_fork(&mut self, parent_pid: u32, exec_path: PathBuf, msg: ESForkEvent) {
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
            println!(
                "{pid} excluded (inherited from {parent_pid}) (exclude paths: {:?}",
                base_info.excluded_by_paths
            );
        }

        self.processes.insert(pid, base_info);
    }

    fn handle_exec(&mut self, pid: u32, msg: ESExecEvent) {
        let Some(info) = self.processes.get_mut(&pid) else {
            log::error!("exec received for unknown pid {pid}");
            return;
        };

        info.exec_path = PathBuf::from(msg.dyld_exec_path);

        // If the path is already excluded, no need to add it again
        if info.excluded_by_paths.contains(&info.exec_path) {
            return;
        }

        // Exclude if path is excluded
        if self.exclude_paths.contains(&info.exec_path) {
            info.excluded_by_paths.insert(info.exec_path.to_owned());
            log::trace!("Excluding {pid} by path: {}", info.exec_path.display());
        }
    }

    fn handle_exit(&mut self, pid: u32) {
        if self.processes.remove(&pid).is_none() {
            log::error!("exit syscall for unknown pid {pid}");
        }
    }
}

/// Obtain a list of all pids
fn list_pids() -> io::Result<Vec<u32>> {
    // SAFETY: Passing in null and 0 returns the number of processes
    let num_pids = unsafe { proc_listallpids(ptr::null_mut(), 0) };
    if num_pids <= 0 {
        return Err(io::Error::last_os_error());
    }
    let num_pids = usize::try_from(num_pids).unwrap();
    let mut pids = vec![0u32; num_pids];

    let buf_sz = (num_pids * std::mem::size_of::<u32>()) as i32;
    // SAFETY: 'pids' is large enough to contain 'num_pids' processes
    let num_pids = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut c_void, buf_sz) };
    if num_pids == -1 {
        return Err(io::Error::last_os_error());
    }

    pids.resize(usize::try_from(num_pids).unwrap(), 0);

    Ok(pids)
}

fn process_path(pid: u32) -> io::Result<PathBuf> {
    let mut buffer = [0u8; libc::MAXPATHLEN as usize];
    // SAFETY: `proc_pidpath` returns at most `buffer.len()` bytes
    let buf_len = unsafe {
        proc_pidpath(
            pid as i32,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len() as u32,
        )
    };
    if buf_len == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(PathBuf::from(
        std::str::from_utf8(&buffer[0..buf_len as usize])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid process path"))?,
    ))
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
    dyld_exec_path: String,
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
    pid: u32,
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
