use std::{
    ffi::OsString,
    fs, io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::{AsRawHandle, RawHandle},
    },
    path::{Path, PathBuf},
    ptr,
    sync::{mpsc as sync_mpsc, Arc},
};
use winapi::{
    self,
    um::{
        fileapi::{GetFileAttributesW, GetFullPathNameW},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        ioapiset::{
            CreateIoCompletionPort, DeviceIoControl, GetQueuedCompletionStatus,
            PostQueuedCompletionStatus,
        },
        minwinbase::OVERLAPPED,
        stringapiset::CompareStringOrdinal,
        winbase::{
            ReadDirectoryChangesW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_FLAG_OVERLAPPED, INFINITE,
        },
        winioctl::FSCTL_GET_REPARSE_POINT,
        winnt::{
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_NOTIFY_CHANGE_DIR_NAME,
            FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_INFORMATION, HANDLE,
            IO_REPARSE_TAG_MOUNT_POINT, IO_REPARSE_TAG_SYMLINK, MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
        },
    },
};

const PATH_MONITOR_COMPLETION_KEY_IGNORE: usize = usize::MAX;

const CSTR_EQUAL: i32 = 2;

const ANYSIZE_ARRAY: usize = 1;
const SYMLINK_FLAG_RELATIVE: u32 = 0x00000001;


// See https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/ca069dad-ed16-42aa-b057-b6b207f447cc.
#[repr(C)]
struct ReparseData {
    tag: u32,
    data_length: u16,
    reserved: i16,
    // Offset to a pathname pointing to the target path.
    sub_name_offset: u16,
    sub_name_length: u16,
    // Offset to a user-displayable pathname.
    print_name_offset: u16,
    print_name_length: u16,
    path_buffer: [u16; ANYSIZE_ARRAY],
}

// See https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/b41f1cbf-10df-4a47-98d4-1c52a833d913.
#[repr(C)]
struct ReparseDataSymlink {
    tag: u32,
    data_length: u16,
    reserved: i16,
    // Offset to a pathname pointing to the target path.
    sub_name_offset: u16,
    sub_name_length: u16,
    // Offset to a user-displayable pathname.
    print_name_offset: u16,
    print_name_length: u16,
    flags: u32,
    path_buffer: [u16; ANYSIZE_ARRAY],
}

fn strip_namespace<P: AsRef<Path>>(path: P) -> PathBuf {
    // \??: symlink to "DosDevices"
    path.as_ref()
        .strip_prefix(r"\\??")
        .map(PathBuf::from)
        .unwrap_or(path.as_ref().to_path_buf())
}

/// Returns the target of a reparse point as an absolute path.
/// If `path` is not a link, `None` is returned.
fn resolve_link<T: AsRef<Path> + Copy>(path: T) -> io::Result<Option<PathBuf>> {
    let mut data_buffer = vec![0u8; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize];

    let mut stripped_path = strip_namespace(path);
    if !stripped_path.starts_with(r"\\?\") {
        stripped_path = Path::new(r"\\?\").join(stripped_path);
    }

    // Note: `file_attributes()` doesn't include all attributes, so we must use GetfileAttributesW.
    let mut u16_path: Vec<u16> = stripped_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();
    let attributes = unsafe { GetFileAttributesW(u16_path.as_mut_ptr()) };

    if (attributes & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
        return Ok(None);
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;

    let mut _bytes_returned = 0u32;

    if unsafe {
        DeviceIoControl(
            file.as_raw_handle() as *mut _,
            FSCTL_GET_REPARSE_POINT,
            ptr::null_mut(),
            0u32,
            data_buffer.as_mut_ptr() as *mut _,
            data_buffer.len() as u32,
            &mut _bytes_returned,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    let reparse_tag = unsafe { &*(data_buffer.as_mut_ptr() as *mut ReparseData) }.tag;
    match reparse_tag {
        IO_REPARSE_TAG_SYMLINK => {
            let reparse_data = unsafe { &*(data_buffer.as_mut_ptr() as *mut ReparseDataSymlink) };
            let parsed_path = unsafe {
                std::slice::from_raw_parts(
                    ((&reparse_data.path_buffer) as *const u16).offset(
                        reparse_data.sub_name_offset as isize / std::mem::size_of::<u16>() as isize,
                    ),
                    reparse_data.sub_name_length as usize / std::mem::size_of::<u16>(),
                )
            };
            let mut path_buf = PathBuf::from(OsString::from_wide(parsed_path));

            if reparse_data.flags & SYMLINK_FLAG_RELATIVE != 0 {
                if let Some(parent) = stripped_path.parent() {
                    let path_buf_os: Vec<u16> = parent
                        .join(path_buf)
                        .into_os_string()
                        .encode_wide()
                        .chain(std::iter::once(0u16))
                        .collect();

                    let mut full_path_buffer = vec![0u16; 2048 / std::mem::size_of::<u16>()];

                    let full_length = loop {
                        let required_length = unsafe {
                            GetFullPathNameW(
                                path_buf_os.as_ptr(),
                                full_path_buffer.len() as u32,
                                full_path_buffer.as_mut_ptr(),
                                ptr::null_mut(),
                            )
                        } as usize;

                        if required_length == 0 {
                            return Err(io::Error::last_os_error());
                        }

                        if required_length > full_path_buffer.len() {
                            full_path_buffer.resize(required_length, 0);
                        } else {
                            break required_length;
                        }
                    };

                    full_path_buffer.resize(full_length, 0);
                    path_buf = PathBuf::from(OsString::from_wide(&full_path_buffer));
                }
            } else {
                path_buf = strip_namespace(path_buf);
            }

            Ok(Some(path_buf))
        }
        IO_REPARSE_TAG_MOUNT_POINT => {
            let reparse_data = unsafe { &*(data_buffer.as_mut_ptr() as *mut ReparseData) };
            let parsed_path = unsafe {
                std::slice::from_raw_parts(
                    ((&reparse_data.path_buffer) as *const u16).offset(
                        reparse_data.sub_name_offset as isize / std::mem::size_of::<u16>() as isize,
                    ),
                    reparse_data.sub_name_length as usize / std::mem::size_of::<u16>(),
                )
            };
            Ok(Some(strip_namespace(PathBuf::from(OsString::from_wide(
                parsed_path,
            )))))
        }
        // unknown reparse tag
        _ => Ok(None),
    }
}

/// The same as [`resolve_all_links`] but for a set of paths.
fn resolve_all_links_multiple<P: AsRef<Path>>(paths: &[P]) -> std::collections::HashSet<PathBuf> {
    let mut monitored_paths = std::collections::HashSet::new();
    for path in paths {
        match resolve_all_links(path) {
            Ok(paths) => monitored_paths.extend(paths),
            Err(error) => {
                log::error!("Failed to identify paths to monitor: {:?}", error);
            }
        }
    }
    monitored_paths
}

/// Returns all links and targets for a given path (including any of its parent directories).
fn resolve_all_links<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
    let mut monitor_paths = vec![path.as_ref().to_path_buf()];
    let mut iter = path.as_ref().components();

    let mut partial_path = PathBuf::new();
    for _ in 0..2 {
        partial_path.push(iter.next().ok_or(io::Error::new(
            io::ErrorKind::Other,
            "path must be absolute",
        ))?);
    }

    for component in &mut iter {
        partial_path.push(component);
        if let Ok(Some(target)) = resolve_link(&partial_path) {
            monitor_paths.extend(resolve_all_links(target.join(iter))?);
            break;
        }
    }

    Ok(monitor_paths)
}

struct DirContext {
    path: PathBuf,
    dir_handle: fs::File,
    buffer: Vec<u32>,
    overlapped: OVERLAPPED,
}

impl DirContext {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<DirContext> {
        let dir_handle = fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OVERLAPPED | FILE_FLAG_BACKUP_SEMANTICS)
            .open(&path)?;
        Ok(DirContext {
            path: path.as_ref().to_path_buf(),
            dir_handle,
            buffer: vec![0u32; 1024],
            overlapped: unsafe { std::mem::zeroed() },
        })
    }

    fn attach_to_io_port(
        &self,
        io_completion_port: &CompletionPort,
        completion_key: usize,
    ) -> io::Result<()> {
        let handle = unsafe {
            CreateIoCompletionPort(
                self.dir_handle.as_raw_handle() as *mut _,
                io_completion_port.as_raw_handle() as *mut _,
                completion_key,
                // num of threads is ignored here
                0,
            )
        };

        if handle == ptr::null_mut() {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn read_directory_changes(&mut self) -> io::Result<()> {
        let mut _bytes_returned = 0;
        if unsafe {
            ReadDirectoryChangesW(
                self.dir_handle.as_raw_handle() as *mut _,
                self.buffer.as_mut_ptr() as *mut _,
                (self.buffer.len() * std::mem::size_of::<u32>()) as u32,
                1,
                FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME,
                &mut _bytes_returned,
                &mut self.overlapped,
                None,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

unsafe impl Send for DirContext {}
unsafe impl Sync for DirContext {}

struct CompletionPort {
    handle: HANDLE,
}

impl CompletionPort {
    // `concurrent_threads`: 0 ==> number of processors
    fn create(concurrent_threads: u32) -> io::Result<Self> {
        let handle = unsafe {
            CreateIoCompletionPort(INVALID_HANDLE_VALUE, ptr::null_mut(), 0, concurrent_threads)
        };
        if handle == ptr::null_mut() {
            return Err(io::Error::last_os_error());
        }
        Ok(CompletionPort { handle })
    }

    fn get_queued_completion_status(&self) -> io::Result<CompletionStatus> {
        let mut result = CompletionStatus {
            bytes_returned: 0,
            completion_key: 0,
            used_overlapped: ptr::null_mut(),
        };

        if unsafe {
            GetQueuedCompletionStatus(
                self.handle,
                &mut result.bytes_returned,
                &mut result.completion_key,
                &mut result.used_overlapped,
                INFINITE,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }

        Ok(result)
    }

    fn post_queued_completion_status(
        &self,
        bytes_transferred: u32,
        completion_key: usize,
        overlapped: *mut OVERLAPPED,
    ) -> io::Result<()> {
        if unsafe {
            PostQueuedCompletionStatus(self.handle, bytes_transferred, completion_key, overlapped)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl AsRawHandle for CompletionPort {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle as *mut _
    }
}

impl Drop for CompletionPort {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

unsafe impl Send for CompletionPort {}
unsafe impl Sync for CompletionPort {}

struct CompletionStatus {
    bytes_returned: u32,
    completion_key: usize,
    used_overlapped: *mut OVERLAPPED,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct StrippedPath {
    prefix: PathBuf,
    tail: Vec<u16>,
}

pub struct PathMonitorHandle {
    port_handle: Arc<CompletionPort>,
    tx: sync_mpsc::Sender<PathMonitorCommand>,
}

impl PathMonitorHandle {
    pub fn set_paths<P: AsRef<Path>>(&self, paths: &[P]) -> io::Result<()> {
        let _ = self.tx.send(PathMonitorCommand::SetPaths(
            paths.iter().map(|p| p.as_ref().to_path_buf()).collect(),
        ));
        self.notify_monitor()
    }

    pub fn shutdown(&self) -> io::Result<()> {
        let _ = self.tx.send(PathMonitorCommand::Shutdown);
        self.notify_monitor()
    }

    fn notify_monitor(&self) -> io::Result<()> {
        self.port_handle.post_queued_completion_status(
            0,
            PATH_MONITOR_COMPLETION_KEY_IGNORE,
            ptr::null_mut(),
        )
    }
}

pub type PathChangeNotifyRx = sync_mpsc::Receiver<()>;

enum PathMonitorCommand {
    Shutdown,
    SetPaths(Vec<PathBuf>),
}

pub struct PathMonitor {
    port_handle: Arc<CompletionPort>,
    dir_contexts: Vec<DirContext>,
    stripped_paths: std::collections::HashSet<StrippedPath>,
}

impl PathMonitor {
    pub fn spawn<P: AsRef<Path>>(
        paths: &[P],
    ) -> io::Result<(PathMonitorHandle, PathChangeNotifyRx)> {
        let port_handle = Arc::new(CompletionPort::create(0)?);
        let mut original_paths: Vec<PathBuf> =
            paths.iter().map(|p| p.as_ref().to_path_buf()).collect();

        let mut resolved_paths = resolve_all_links_multiple(&original_paths);
        let stripped_paths = resolved_paths
            .iter()
            .filter_map(|p| Self::strip_path(p).ok())
            .collect();

        let mut monitor = Self {
            port_handle: port_handle.clone(),
            dir_contexts: vec![],
            stripped_paths,
        };

        monitor.update_directory_contexts()?;

        let (cmd_tx, cmd_rx) = sync_mpsc::channel();
        let (notify_tx, notify_rx) = sync_mpsc::channel();

        std::thread::spawn(move || {
            loop {
                let mut stop_monitor = false;
                while let Some(cmd) = cmd_rx.try_iter().next() {
                    match cmd {
                        PathMonitorCommand::Shutdown => {
                            stop_monitor = true;
                            break;
                        }
                        PathMonitorCommand::SetPaths(new_paths) => {
                            original_paths = new_paths;
                            resolved_paths = resolve_all_links_multiple(&original_paths);
                            monitor.stripped_paths = resolved_paths
                                .iter()
                                .filter_map(|p| Self::strip_path(p).ok())
                                .collect();
                            if let Err(error) = monitor.update_directory_contexts() {
                                log::error!("Failed to set open new directory handles: {}", error);
                                stop_monitor = true;
                                break;
                            }
                        }
                    }
                }
                if stop_monitor {
                    break;
                }

                let result = match monitor.port_handle.get_queued_completion_status() {
                    Ok(result) if result.completion_key == PATH_MONITOR_COMPLETION_KEY_IGNORE => {
                        continue
                    }
                    Ok(result) => result,
                    Err(error) => {
                        log::error!(
                            "GetQueuedCompletionStatus failed: {:?}",
                            error.raw_os_error()
                        );
                        break;
                    }
                };

                if result.completion_key >= monitor.dir_contexts.len() {
                    log::debug!("Ignoring out-of-bounds completion key");
                    continue;
                }

                if result.bytes_returned == 0 {
                    log::debug!("Change event buffer is empty");
                }

                if let Err(error) =
                    monitor.dir_contexts[result.completion_key].read_directory_changes()
                {
                    log::error!("Failed to queue new directory change event: {}", error);
                    break;
                }

                if result.bytes_returned == 0 || result.used_overlapped == ptr::null_mut() {
                    continue;
                }

                let mut info = monitor.dir_contexts[result.completion_key].buffer.as_ptr()
                    as *const FILE_NOTIFY_INFORMATION;
                let mut changed = false;
                loop {
                    let current_field = unsafe { &*info };

                    let file_name = unsafe {
                        std::slice::from_raw_parts(
                            current_field.FileName.as_ptr(),
                            current_field.FileNameLength as usize / std::mem::size_of::<u16>(),
                        )
                    };

                    for path in &monitor.stripped_paths {
                        if path.prefix != monitor.dir_contexts[result.completion_key].path() {
                            continue;
                        }
                        if path.tail.len() < file_name.len() {
                            continue;
                        }
                        let cmp_status = unsafe {
                            CompareStringOrdinal(
                                path.tail.as_ptr(),
                                file_name.len() as i32,
                                file_name.as_ptr(),
                                file_name.len() as i32,
                                1,
                            )
                        };
                        match cmp_status {
                            CSTR_EQUAL => {
                                changed = true;
                                break;
                            }
                            0 => log::error!("Bug: CompareStringOrdinal failed"),
                            _ => (),
                        }
                    }

                    if changed || current_field.NextEntryOffset == 0 {
                        break;
                    }
                    info =
                        unsafe { (info as *mut u8).offset(current_field.NextEntryOffset as isize) }
                            as *const FILE_NOTIFY_INFORMATION;
                }
                if changed {
                    let new_resolved_paths = resolve_all_links_multiple(&original_paths);
                    if new_resolved_paths != resolved_paths {
                        resolved_paths = new_resolved_paths;
                        monitor.stripped_paths = resolved_paths
                            .iter()
                            .filter_map(|p| Self::strip_path(p).ok())
                            .collect();
                        if let Err(error) = monitor.update_directory_contexts() {
                            log::error!("Failed to set open new directory handles: {}", error);
                            break;
                        }
                        let _ = notify_tx.send(());
                    }
                }
            }
            log::debug!("Shutting down reparse point monitor");
        });

        Ok((
            PathMonitorHandle {
                port_handle,
                tx: cmd_tx,
            },
            notify_rx,
        ))
    }

    fn update_directory_contexts(&mut self) -> io::Result<()> {
        // Remove paths we no longer need to monitor
        let len = self.dir_contexts.len();
        for i in (0..len).rev() {
            if !self
                .stripped_paths
                .iter()
                .any(|p| p.prefix == self.dir_contexts[i].path)
            {
                self.dir_contexts.remove(i);
            }
        }

        // Add new paths to monitor
        for path in &self.stripped_paths {
            if self
                .dir_contexts
                .iter()
                .any(|ctx| path.prefix == ctx.path())
            {
                continue;
            }

            let mut ctx = match DirContext::new(&path.prefix) {
                Ok(ctx) => ctx,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    log::warn!(
                        "Not monitoring reparse points on {} since it does not exist",
                        path.prefix.to_string_lossy()
                    );
                    continue;
                }
                Err(error) => return Err(error),
            };
            let index = self.dir_contexts.len();
            ctx.attach_to_io_port(&self.port_handle, index)?;
            ctx.read_directory_changes()?;
            self.dir_contexts.push(ctx);
        }

        Ok(())
    }

    fn strip_path(path: &PathBuf) -> io::Result<StrippedPath> {
        let mut iter = path.components();
        let prefix = iter.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path is missing prefix",
        ))?;
        let prefix = Path::new(&prefix).join(iter.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path is missing root",
        ))?);

        Ok(StrippedPath {
            prefix: prefix.clone(),
            tail: iter.as_path().as_os_str().encode_wide().collect(),
        })
    }
}
