use libc::{proc_bsdinfo, proc_listallpids, proc_pidinfo, proc_pidpath, PROC_PIDTBSDINFO};
use serde::Deserialize;
use std::{
    collections::HashMap,
    ffi::c_void,
    io, mem,
    path::PathBuf,
    process::Stdio,
    ptr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
    task::JoinHandle,
};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to start eslogger listener
    #[error("Failed to start eslogger")]
    StartMonitor(#[source] io::Error),
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
    proc: Child,
    parser_task: Option<JoinHandle<()>>,
    states: ProcessStates,
}

impl ProcessMonitor {
    pub async fn spawn() -> Result<ProcessMonitorHandle, Error> {
        let states = ProcessStates::new()?;

        let excluded_paths = vec![];
        states.exclude_paths(excluded_paths);

        let mut cmd = tokio::process::Command::new("/usr/bin/eslogger");
        cmd.args(["exec", "fork", "exit"])
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut proc = cmd.spawn().map_err(Error::StartMonitor)?;

        let stdout = proc.stdout.take().unwrap();

        let states_clone = states.clone();

        let parser_task = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
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

        Ok(ProcessMonitorHandle {
            proc,
            parser_task: Some(parser_task),
            states,
        })
    }
}

impl ProcessMonitorHandle {
    pub async fn shutdown(&mut self) {
        log::debug!("Stopping process monitor");

        let Some(parser_task) = self.parser_task.take() else {
            return;
        };

        if let Err(error) = self.proc.kill().await {
            log::error!("Failed to kill eslogger: {error}");
        }
        if tokio::time::timeout(SHUTDOWN_TIMEOUT, parser_task)
            .await
            .is_err()
        {
            log::error!("Failed to wait for ST process handler");
        }
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
    exclude_paths: Vec<PathBuf>,
}

impl ProcessStates {
    /// Initialize process states
    fn new() -> Result<Self, Error> {
        let mut states = InnerProcessStates {
            processes: HashMap::new(),
            exclude_paths: vec![],
        };

        let procs = list_pids().map_err(Error::InitializePids)?;

        for pid in procs {
            let path = process_path(pid).map_err(|error| Error::FindProcessPath(error, pid))?;
            let ppid = process_info(pid)
                .map(|info| info.pbi_ppid)
                .unwrap_or_else(|error| {
                    log::error!("Failed to obtain parent pid for {pid}: {error}");
                    0
                });

            states
                .processes
                .insert(pid, ProcessInfo::included(path, ppid));
        }

        Ok(ProcessStates {
            inner: Arc::new(Mutex::new(states)),
        })
    }

    pub fn exclude_paths(&self, paths: Vec<PathBuf>) {
        let mut inner = self.inner.lock().unwrap();

        for (_pid, info) in &mut inner.processes {
            // Remove no-longer excluded paths from exclusion list
            let mut new_exclude_paths: Vec<_> = info
                .excluded_by_paths
                .iter()
                .filter(|old_path| paths.contains(old_path))
                .cloned()
                .collect();

            // Check if own path is excluded
            if paths.contains(&info.exec_path) {
                if !new_exclude_paths.contains(&info.exec_path) {
                    new_exclude_paths.push(info.exec_path.to_owned());
                }
            }

            info.excluded_by_paths = Arc::from(new_exclude_paths);
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
            ESEvent::Exit(_) => self.handle_exit(pid),
        }
    }

    // For new processes, inherit all exclusion state from the parent, if there is one.
    // Otherwise, look up excluded paths
    fn handle_fork(&mut self, ppid: u32, exec_path: PathBuf, msg: ESForkEvent) {
        let pid = msg.child.audit_token.pid;

        if self.processes.contains_key(&pid) {
            log::error!("Conflicting pid! State already contains {pid}");
        }

        // Inherit exclusion status from parent
        let mut base_info = match self.processes.get(&ppid) {
            Some(parent_info) => parent_info.to_owned(),
            None => {
                log::error!("{pid}: Unknown parent pid {ppid}!");
                ProcessInfo::included(exec_path, ppid)
            }
        };

        // no exec yet; only pid and ppid change
        base_info.ppid = ppid;

        if base_info.is_excluded() {
            println!(
                "{pid} excluded (inherited from {ppid}) (exclude paths: {:?}",
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

        if info.excluded_by_paths.contains(&info.exec_path) {
            return;
        }

        // Exclude if path is excluded
        if self.exclude_paths.contains(&info.exec_path) {
            let mut new_paths = info.excluded_by_paths.to_vec();
            new_paths.push(info.exec_path.to_owned());
            info.excluded_by_paths = Arc::from(new_paths);

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
    let num_pids = unsafe { proc_listallpids(ptr::null_mut(), 0) };
    if num_pids <= 0 {
        return Err(io::Error::last_os_error());
    }
    let mut pids = vec![0u32; usize::try_from(num_pids).unwrap()];

    let buf_sz = (u32::BITS as usize / 8 * (num_pids as usize)) as i32;
    let num_pids = unsafe { proc_listallpids(pids.as_mut_ptr() as *mut c_void, buf_sz) };
    if num_pids == -1 {
        return Err(io::Error::last_os_error());
    }

    pids.resize(usize::try_from(num_pids).unwrap(), 0);

    Ok(pids)
}

fn process_path(pid: u32) -> io::Result<PathBuf> {
    let mut buffer = [0u8; libc::MAXPATHLEN as usize];
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

fn process_info(pid: u32) -> io::Result<proc_bsdinfo> {
    let mut info: proc_bsdinfo = unsafe { std::mem::zeroed() };

    let result = unsafe {
        proc_pidinfo(
            pid as i32,
            PROC_PIDTBSDINFO,
            0,
            &mut info as *mut _ as *mut c_void,
            mem::size_of::<proc_bsdinfo>() as i32,
        )
    };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(info)
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    exec_path: PathBuf,
    ppid: u32,
    // inherited: bool,
    excluded_by_pid: bool,
    excluded_by_paths: Arc<[PathBuf]>,
}

impl ProcessInfo {
    fn included(exec_path: PathBuf, ppid: u32) -> Self {
        ProcessInfo {
            exec_path,
            ppid,
            excluded_by_pid: false,
            excluded_by_paths: Arc::from([]),
        }
    }

    fn is_excluded(&self) -> bool {
        self.excluded_by_pid || !self.excluded_by_paths.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ESForkChild {
    audit_token: ESAuditToken,
}

#[derive(Debug, Deserialize)]
struct ESForkEvent {
    child: ESForkChild,
}

#[derive(Debug, Deserialize)]
struct ESExecEvent {
    dyld_exec_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ESEvent {
    Fork(ESForkEvent),
    Exec(ESExecEvent),
    Exit(serde_json::Value),
}

#[derive(Debug, Deserialize)]
struct ESExecutable {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ESAuditToken {
    pid: u32,
}

#[derive(Debug, Deserialize)]
struct ESProcess {
    audit_token: ESAuditToken,
    executable: ESExecutable,
}

#[derive(Debug, Deserialize)]
struct ESMessage {
    event: ESEvent,
    process: ESProcess,
}
