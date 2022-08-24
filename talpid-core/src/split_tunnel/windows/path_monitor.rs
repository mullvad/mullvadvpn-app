use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs, io, mem,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::{AsRawHandle, RawHandle},
    },
    path::{Path, PathBuf},
    pin::Pin,
    ptr,
    sync::{mpsc as sync_mpsc, Arc},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_NOT_FOUND, ERROR_OPERATION_ABORTED, ERROR_PATH_NOT_FOUND,
        ERROR_UNRECOGNIZED_VOLUME, HANDLE, INVALID_HANDLE_VALUE,
    },
    Globalization::CompareStringOrdinal,
    Storage::FileSystem::{
        GetFileAttributesW, GetFullPathNameW, ReadDirectoryChangesW, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_OVERLAPPED,
        FILE_NOTIFY_CHANGE_ATTRIBUTES, FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME,
        FILE_NOTIFY_INFORMATION,
    },
    System::{
        Ioctl::FSCTL_GET_REPARSE_POINT,
        SystemServices::{IO_REPARSE_TAG_MOUNT_POINT, IO_REPARSE_TAG_SYMLINK},
        WindowsProgramming::INFINITE,
        IO::{
            CancelIoEx, CreateIoCompletionPort, DeviceIoControl, GetQueuedCompletionStatus,
            PostQueuedCompletionStatus, OVERLAPPED,
        },
    },
};

const SHUTDOWN_POLL_TIMEOUT: Duration = Duration::from_millis(500);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const PATH_MONITOR_COMPLETION_KEY_IGNORE: usize = usize::MAX;

const CSTR_EQUAL: i32 = 2;

const ANYSIZE_ARRAY: usize = 1;
const MAXIMUM_REPARSE_DATA_BUFFER_SIZE: u32 = 16384;
const SYMLINK_FLAG_RELATIVE: u32 = 0x00000001;

// See https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/c3a420cb-8a72-4adf-87e8-eee95379d78f.
#[repr(C)]
struct ReparseData {
    tag: u32,
    data_length: u16,
    reserved: u16,
    data: [u8; ANYSIZE_ARRAY],
}

// See https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/ca069dad-ed16-42aa-b057-b6b207f447cc.
#[repr(C)]
struct ReparseDataMountPoint {
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

macro_rules! get_reparse_path {
    ($tag_type:ident, $data:ident) => {{
        let reparse_data = &*($data.as_ptr() as *const $tag_type);
        let last_offset = reparse_data.sub_name_offset as usize
            + reparse_data.sub_name_length as usize
            + memoffset::offset_of!($tag_type, path_buffer);

        if last_offset > $data.len() {
            log::error!("Ignoring mount point with out-of-bounds index");
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "link indices out-of-bounds",
            ))
        } else {
            let path_buffer = (&reparse_data.path_buffer) as *const u16;
            let parsed_path = std::slice::from_raw_parts(
                path_buffer.offset(
                    (reparse_data.sub_name_offset as usize / mem::size_of::<u16>()) as isize,
                ),
                reparse_data.sub_name_length as usize / mem::size_of::<u16>(),
            );
            Ok::<PathBuf, io::Error>(PathBuf::from(OsString::from_wide(parsed_path)))
        }
    }};
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
    let mut u16_path: Vec<u16> = osstr_to_wide(&stripped_path);
    let attributes = unsafe { GetFileAttributesW(u16_path.as_mut_ptr()) };

    if (attributes & FILE_ATTRIBUTE_REPARSE_POINT) == 0 {
        return Ok(None);
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;

    let mut bytes_returned = 0u32;

    if unsafe {
        DeviceIoControl(
            file.as_raw_handle() as HANDLE,
            FSCTL_GET_REPARSE_POINT,
            ptr::null_mut(),
            0u32,
            data_buffer.as_mut_ptr() as *mut _,
            data_buffer.len() as u32,
            &mut bytes_returned,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    if (bytes_returned as usize) < mem::size_of::<ReparseDataMountPoint>() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid reparse point data",
        ));
    }

    let reparse_tag = unsafe { &*(data_buffer.as_ptr() as *const ReparseData) }.tag;
    match reparse_tag {
        IO_REPARSE_TAG_SYMLINK => {
            let is_relative = unsafe { &*(data_buffer.as_ptr() as *const ReparseDataSymlink) }
                .flags
                & SYMLINK_FLAG_RELATIVE
                != 0;
            let mut path_buf = unsafe { get_reparse_path!(ReparseDataSymlink, data_buffer) }?;

            if is_relative {
                if let Some(parent) = stripped_path.parent() {
                    path_buf = get_full_path_name(parent.join(path_buf))?;
                }
            } else {
                path_buf = strip_namespace(path_buf);
            }

            Ok(Some(path_buf))
        }
        IO_REPARSE_TAG_MOUNT_POINT => {
            let path_buf = unsafe { get_reparse_path!(ReparseDataMountPoint, data_buffer) }?;
            Ok(Some(strip_namespace(path_buf)))
        }
        // unknown reparse tag
        _ => Ok(None),
    }
}

/// The same as [`resolve_all_links`] but for a set of paths.
fn resolve_all_links_multiple<P: AsRef<Path>>(paths: &[P]) -> HashSet<PathBuf> {
    let mut monitored_paths = HashSet::new();
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
    partial_path.push(iter.next().ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "path is missing prefix",
    ))?);
    partial_path.push(iter.next().ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "path is missing root",
    ))?);

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
    buffer: Vec<u8>,
    overlapped: Pin<Box<OVERLAPPED>>,
    _io_completion_port: Arc<CompletionPort>,
}

impl DirContext {
    fn new<P: AsRef<Path>>(
        path: P,
        io_completion_port: Arc<CompletionPort>,
        completion_key: usize,
    ) -> io::Result<DirContext> {
        let dir_handle = fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OVERLAPPED | FILE_FLAG_BACKUP_SEMANTICS)
            .open(&path)?;

        let handle = unsafe {
            CreateIoCompletionPort(
                dir_handle.as_raw_handle() as HANDLE,
                io_completion_port.as_raw_handle() as HANDLE,
                completion_key,
                // num of threads is ignored here
                0,
            )
        };

        if handle == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(DirContext {
            path: path.as_ref().to_path_buf(),
            dir_handle,
            buffer: vec![0u8; 16 * 1024],
            overlapped: Box::pin(unsafe { mem::zeroed() }),
            _io_completion_port: io_completion_port,
        })
    }

    fn read_directory_changes(&mut self) -> io::Result<()> {
        let mut _bytes_returned = 0;
        if unsafe {
            ReadDirectoryChangesW(
                self.dir_handle.as_raw_handle() as HANDLE,
                self.buffer.as_mut_ptr() as *mut _,
                self.buffer.len() as u32,
                1,
                FILE_NOTIFY_CHANGE_FILE_NAME
                    | FILE_NOTIFY_CHANGE_DIR_NAME
                    | FILE_NOTIFY_CHANGE_ATTRIBUTES,
                &mut _bytes_returned,
                &mut *self.overlapped,
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

    /// Try to cancel a request. On success, return whether a request was cancelled.
    fn cancel_io(&mut self) -> io::Result<bool> {
        if unsafe { CancelIoEx(self.dir_handle.as_raw_handle() as HANDLE, ptr::null_mut()) } == 0 {
            match io::Error::last_os_error() {
                _error if _error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => Ok(false),
                error => Err(error),
            }
        } else {
            Ok(true)
        }
    }
}

impl Drop for DirContext {
    fn drop(&mut self) {
        if let Err(error) = self.cancel_io() {
            log::error!("Failed to cancel pending file I/O request: {}", error);
        }
    }
}

unsafe impl Send for DirContext {}
unsafe impl Sync for DirContext {}

struct CompletionStatus {
    bytes_returned: u32,
    completion_key: usize,
    used_overlapped: *mut OVERLAPPED,
}

struct CompletionPort {
    handle: HANDLE,
}

impl CompletionPort {
    // `concurrent_threads`: 0 ==> number of processors
    fn create(concurrent_threads: u32) -> io::Result<Self> {
        let handle =
            unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, 0, 0, concurrent_threads) };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(CompletionPort { handle })
    }

    fn get_queued_completion_status(
        &self,
    ) -> Result<CompletionStatus, (io::Error, CompletionStatus)> {
        self.get_queued_completion_status_timeout(INFINITE)
    }

    fn get_queued_completion_status_timeout(
        &self,
        timeout: u32,
    ) -> Result<CompletionStatus, (io::Error, CompletionStatus)> {
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
                timeout,
            )
        } == 0
        {
            return Err((io::Error::last_os_error(), result));
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

#[derive(Clone, Hash, PartialEq, Eq)]
struct StrippedPath {
    /// The volume that the path is on. For `C:\a\b\c`, this would be for `C:\`.
    prefix: PathBuf,
    /// The remainder of the path. For `C:\a\b\c`, this would be for `a\b\c`.
    tail: Vec<u16>,
}

impl StrippedPath {
    fn new(path: &PathBuf) -> io::Result<StrippedPath> {
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
            tail: osstr_to_wide(iter.as_path()),
        })
    }
}

#[derive(Clone)]
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

    pub fn refresh(&self) -> io::Result<()> {
        let _ = self.tx.send(PathMonitorCommand::Refresh);
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

enum PathMonitorCommand {
    SetPaths(Vec<PathBuf>),
    Refresh,
    Shutdown,
}

pub struct PathMonitor {
    port_handle: Arc<CompletionPort>,
    dir_contexts: Vec<DirContext>,
    discarded_contexts: Vec<DirContext>,
    stripped_paths: HashSet<StrippedPath>,
}

impl PathMonitor {
    pub fn spawn(update_notify_tx: sync_mpsc::Sender<()>) -> io::Result<PathMonitorHandle> {
        let port_handle = Arc::new(CompletionPort::create(0)?);
        let mut original_paths: Vec<PathBuf> = vec![];

        let mut monitor = Self {
            port_handle: port_handle.clone(),
            dir_contexts: vec![],
            discarded_contexts: vec![],
            stripped_paths: HashSet::new(),
        };

        let (cmd_tx, cmd_rx) = sync_mpsc::channel();

        std::thread::spawn(move || {
            loop {
                if !monitor.service_commands(&mut original_paths, &cmd_rx) {
                    break;
                }
                match monitor.handle_next_completion_packet() {
                    Ok(true) => match monitor.update_paths(&original_paths, false) {
                        Ok(true) => {
                            let _ = update_notify_tx.send(());
                        }
                        Ok(false) => (),
                        Err(_) => break,
                    },
                    Ok(false) => (),
                    Err(error) => {
                        log::error!("handle_next_completion_packet failed: {}", error);
                        break;
                    }
                }
            }
            log::debug!("Shutting down reparse point monitor");

            monitor.abort_all_requests();
        });

        Ok(PathMonitorHandle {
            port_handle,
            tx: cmd_tx,
        })
    }

    fn service_commands(
        &mut self,
        original_paths: &mut Vec<PathBuf>,
        cmd_rx: &sync_mpsc::Receiver<PathMonitorCommand>,
    ) -> bool {
        while let Some(cmd) = cmd_rx.try_iter().next() {
            match cmd {
                PathMonitorCommand::Shutdown => {
                    return false;
                }
                PathMonitorCommand::SetPaths(new_paths) => {
                    *original_paths = new_paths;
                    return !self.update_paths(&original_paths, false).is_err();
                }
                PathMonitorCommand::Refresh => {
                    return !self.update_paths(&original_paths, true).is_err();
                }
            }
        }
        true
    }

    fn update_paths(&mut self, unresolved_paths: &[PathBuf], force: bool) -> Result<bool, ()> {
        let resolved_paths = resolve_all_links_multiple(unresolved_paths);
        let new_stripped_paths = resolved_paths
            .iter()
            .filter_map(|p| StrippedPath::new(p).ok())
            .collect();
        if force || new_stripped_paths != self.stripped_paths {
            self.stripped_paths = new_stripped_paths;
            if let Err(error) = self.update_directory_contexts() {
                log::error!("Failed to open new directory handles: {}", error);
                return Err(());
            }
            return Ok(true);
        }
        Ok(false)
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
                let mut removed_ctx = self.dir_contexts.remove(i);
                match removed_ctx.cancel_io() {
                    Ok(true) => self.discarded_contexts.push(removed_ctx),
                    Err(error) => {
                        log::error!("Failed to cancel pending I/O for dir context: {}", error);
                        mem::forget(removed_ctx)
                    }
                    Ok(false) => (),
                }
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

            let index = self.dir_contexts.len();
            let mut ctx = match DirContext::new(&path.prefix, self.port_handle.clone(), index) {
                Ok(ctx) => ctx,
                Err(error) => {
                    match error.raw_os_error().map(|code| code as u32) {
                        Some(ERROR_NOT_FOUND)
                        | Some(ERROR_PATH_NOT_FOUND)
                        | Some(ERROR_UNRECOGNIZED_VOLUME) => {
                            log::trace!(
                                "Not monitoring reparse points on {} since it does not exist",
                                path.prefix.to_string_lossy()
                            );
                        }
                        _ => return Err(error),
                    }
                    continue;
                }
            };
            ctx.read_directory_changes()?;
            self.dir_contexts.push(ctx);
        }

        Ok(())
    }

    fn handle_next_completion_packet(&mut self) -> io::Result<bool> {
        let result = match self.port_handle.get_queued_completion_status() {
            Ok(result) if result.completion_key == PATH_MONITOR_COMPLETION_KEY_IGNORE => {
                return Ok(false);
            }
            Err((error, status)) => {
                self.free_discarded_context(status.used_overlapped);
                if error.raw_os_error() != Some(ERROR_OPERATION_ABORTED as i32) {
                    log::error!("GetQueuedCompletionStatus failed: {:?}", error);
                    return Err(error);
                }
                return Ok(false);
            }
            Ok(result) => result,
        };

        if self.free_discarded_context(result.used_overlapped) {
            return Ok(false);
        }

        let ctx_index = self
            .find_context(result.used_overlapped)
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                "received I/O completion packet without an associated DirContext",
            ))?;

        let changed = if result.bytes_returned == 0 {
            log::trace!("Change event buffer is empty");
            false
        } else {
            self.process_file_notification(&self.dir_contexts[ctx_index])?
        };

        if let Err(error) = self.dir_contexts[ctx_index].read_directory_changes() {
            log::error!("Failed to queue new directory change event: {}", error);
            return Err(error);
        }

        Ok(changed)
    }

    /// Find the index of the `DirContext` that owns the `OVERLAPPED` object, or None.
    fn find_context(&self, overlapped: *const OVERLAPPED) -> Option<usize> {
        if overlapped == ptr::null_mut() {
            return None;
        }
        for i in 0..self.dir_contexts.len() {
            if ((&*self.dir_contexts[i].overlapped) as *const _) == overlapped {
                return Some(i);
            }
        }
        None
    }

    /// Remove the element in `discarded_contexts` that owns the `OVERLAPPED` object, if it exists.
    fn free_discarded_context(&mut self, overlapped: *const OVERLAPPED) -> bool {
        if overlapped == ptr::null_mut() {
            return false;
        }
        let mut was_discarded = false;
        self.discarded_contexts.retain(|ctx| {
            if ((&*ctx.overlapped) as *const _) != overlapped {
                true
            } else {
                was_discarded = true;
                false
            }
        });
        was_discarded
    }

    fn process_file_notification(&self, dir_context: &DirContext) -> io::Result<bool> {
        let mut info = dir_context.buffer.as_ptr() as *const FILE_NOTIFY_INFORMATION;
        loop {
            let current_field = unsafe { &*info };

            let file_name = unsafe {
                std::slice::from_raw_parts(
                    current_field.FileName.as_ptr(),
                    current_field.FileNameLength as usize / mem::size_of::<u16>(),
                )
            };

            for path in &self.stripped_paths {
                if path.prefix != dir_context.path() {
                    continue;
                }
                if path.tail.len() <= file_name.len() {
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
                    CSTR_EQUAL => return Ok(true),
                    0 => log::error!("Bug: CompareStringOrdinal failed"),
                    _ => (),
                }
            }

            if current_field.NextEntryOffset == 0 {
                break;
            }
            info = unsafe { (info as *mut u8).offset(current_field.NextEntryOffset as isize) }
                as *const FILE_NOTIFY_INFORMATION;
        }
        Ok(false)
    }

    /// Cancel all requests and give the cancelled operations some time to complete.
    fn abort_all_requests(&mut self) {
        let mut contexts = vec![];
        for mut ctx in self
            .dir_contexts
            .drain(..)
            .chain(self.discarded_contexts.drain(..))
        {
            match ctx.cancel_io() {
                Ok(true) => contexts.push(ctx),
                Ok(false) => (),
                Err(error) => {
                    log::error!("Failed to cancel pending I/O request: {}", error);
                    mem::forget(ctx);
                }
            }
        }

        let time = Instant::now();
        while !contexts.is_empty() {
            if time.elapsed() >= SHUTDOWN_TIMEOUT {
                log::error!("Timeout while cancelling I/O requests");
                mem::forget(contexts);
                return;
            }

            let result = match self
                .port_handle
                .get_queued_completion_status_timeout(SHUTDOWN_POLL_TIMEOUT.as_millis() as u32)
            {
                Ok(result) => result,
                Err((error, result)) => {
                    if error.raw_os_error() != Some(ERROR_OPERATION_ABORTED as i32) {
                        log::error!("GetQueuedCompletionStatus failed: {:?}", error);
                        if result.used_overlapped == ptr::null_mut() {
                            continue;
                        }
                    }
                    result
                }
            };
            contexts.retain(|ctx| ((&*ctx.overlapped) as *const _) != result.used_overlapped);
        }
    }
}

fn get_full_path_name<T: AsRef<OsStr>>(path: T) -> io::Result<PathBuf> {
    let path_buf_os: Vec<u16> = osstr_to_wide(path);
    let mut full_path_buffer = vec![0u16; 2048 / mem::size_of::<u16>()];

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
    Ok(PathBuf::from(OsString::from_wide(&full_path_buffer)))
}

/// Converts an `OsStr` to a null-terminated UTF-16 string.
fn osstr_to_wide<T: AsRef<OsStr>>(string: T) -> Vec<u16> {
    string
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect()
}
