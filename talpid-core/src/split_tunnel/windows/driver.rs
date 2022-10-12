use super::windows::{
    get_device_path, get_process_creation_time, get_process_device_path, open_process, Event,
    Overlapped, ProcessAccess, ProcessSnapshot,
};
use bitflags::bitflags;
use memoffset::offset_of;
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io,
    mem::{self, size_of, MaybeUninit},
    net::{Ipv4Addr, Ipv6Addr},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::{AsRawHandle, RawHandle},
    },
    path::Path,
    ptr,
    time::Duration,
};
use talpid_types::ErrorExt;
use windows_sys::Win32::{
    Foundation::{
        ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_INVALID_PARAMETER, ERROR_IO_PENDING,
        HANDLE, NTSTATUS, WAIT_ABANDONED, WAIT_ABANDONED_0, WAIT_FAILED, WAIT_OBJECT_0,
    },
    Networking::WinSock::{IN6_ADDR, IN_ADDR},
    Storage::FileSystem::FILE_FLAG_OVERLAPPED,
    System::{
        Diagnostics::ToolHelp::TH32CS_SNAPPROCESS,
        Ioctl::{FILE_ANY_ACCESS, METHOD_BUFFERED, METHOD_NEITHER},
        Threading::{WaitForMultipleObjects, WaitForSingleObject},
        WindowsProgramming::INFINITE,
        IO::{DeviceIoControl, GetOverlappedResult, OVERLAPPED},
    },
};

const DRIVER_SYMBOLIC_NAME: &str = "\\\\.\\MULLVADSPLITTUNNEL";
const ST_DEVICE_TYPE: u32 = 0x8000;

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    device_type << 16 | access << 14 | function << 2 | method
}

#[repr(u32)]
#[allow(dead_code)]
pub enum DriverIoctlCode {
    Initialize = ctl_code(ST_DEVICE_TYPE, 1, METHOD_NEITHER, FILE_ANY_ACCESS),
    DequeEvent = ctl_code(ST_DEVICE_TYPE, 2, METHOD_BUFFERED, FILE_ANY_ACCESS),
    RegisterProcesses = ctl_code(ST_DEVICE_TYPE, 3, METHOD_BUFFERED, FILE_ANY_ACCESS),
    RegisterIpAddresses = ctl_code(ST_DEVICE_TYPE, 4, METHOD_BUFFERED, FILE_ANY_ACCESS),
    GetIpAddresses = ctl_code(ST_DEVICE_TYPE, 5, METHOD_BUFFERED, FILE_ANY_ACCESS),
    SetConfiguration = ctl_code(ST_DEVICE_TYPE, 6, METHOD_BUFFERED, FILE_ANY_ACCESS),
    GetConfiguration = ctl_code(ST_DEVICE_TYPE, 7, METHOD_BUFFERED, FILE_ANY_ACCESS),
    ClearConfiguration = ctl_code(ST_DEVICE_TYPE, 8, METHOD_NEITHER, FILE_ANY_ACCESS),
    GetState = ctl_code(ST_DEVICE_TYPE, 9, METHOD_BUFFERED, FILE_ANY_ACCESS),
    QueryProcess = ctl_code(ST_DEVICE_TYPE, 10, METHOD_BUFFERED, FILE_ANY_ACCESS),
    Reset = ctl_code(ST_DEVICE_TYPE, 11, METHOD_NEITHER, FILE_ANY_ACCESS),
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum DriverState {
    // Default state after being loaded.
    None = 0,
    // DriverEntry has completed successfully.
    // Basically only driver and device objects are created at this point.
    Started = 1,
    // All subsystems are initialized.
    Initialized = 2,
    // User mode has registered all processes in the system.
    Ready = 3,
    // IP addresses are registered.
    // A valid configuration is registered.
    Engaged = 4,
    // Driver is unloading.
    Terminating = 5,
}

#[derive(err_derive::Error, Debug)]
#[error(display = "Unknown driver state: {}", _0)]
pub struct UnknownDriverState(u64);

impl TryFrom<u64> for DriverState {
    type Error = UnknownDriverState;

    fn try_from(state: u64) -> Result<Self, Self::Error> {
        use DriverState::*;

        match state {
            e if e == None as u64 => Ok(None),
            e if e == Started as u64 => Ok(Started),
            e if e == Initialized as u64 => Ok(Initialized),
            e if e == Ready as u64 => Ok(Ready),
            e if e == Engaged as u64 => Ok(Engaged),
            e if e == Terminating as u64 => Ok(Terminating),
            other => Err(UnknownDriverState(other)),
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum EventId {
    StartSplittingProcess = 0,
    StopSplittingProcess,

    // ErrorFlag = 0x80000000,
    ErrorStartSplittingProcess = 0x80000001,
    ErrorStopSplittingProcess,

    ErrorMessage,
}

#[derive(err_derive::Error, Debug)]
#[error(display = "Unknown event id: {}", _0)]
pub struct UnknownEventId(u32);

impl TryFrom<u32> for EventId {
    type Error = UnknownEventId;

    fn try_from(event: u32) -> Result<Self, Self::Error> {
        use EventId::*;

        match event {
            e if e == StartSplittingProcess as u32 => Ok(StartSplittingProcess),
            e if e == StopSplittingProcess as u32 => Ok(StopSplittingProcess),
            e if e == ErrorStartSplittingProcess as u32 => Ok(ErrorStartSplittingProcess),
            e if e == ErrorStopSplittingProcess as u32 => Ok(ErrorStopSplittingProcess),
            e if e == ErrorMessage as u32 => Ok(ErrorMessage),
            other => Err(UnknownEventId(other)),
        }
    }
}

pub enum EventBody {
    SplittingEvent {
        process_id: usize,
        reason: SplittingChangeReason,
        image: OsString,
    },
    SplittingError {
        process_id: usize,
        image: OsString,
    },
    ErrorMessage {
        status: NTSTATUS,
        message: OsString,
    },
}

bitflags! {
    pub struct SplittingChangeReason: u32 {
        const BY_INHERITANCE = 1;
        const BY_CONFIG = 2;
        const PROCESS_ARRIVING = 4;
        const PROCESS_DEPARTING = 8;
    }
}

pub struct DeviceHandle {
    handle: fs::File,
}

unsafe impl Sync for DeviceHandle {}
unsafe impl Send for DeviceHandle {}

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum DeviceHandleError {
    /// Failed to connect because there's no such device
    #[error(display = "Failed to connect to driver, no such device. \
            The driver is probably not loaded")]
    ConnectionFailed,

    /// Failed to connect because the connection was denied
    #[error(display = "Failed to connect to driver, connection denied. \
            The exclusive connection is probably hogged")]
    ConnectionDenied,

    /// Failed to connect to driver
    #[error(display = "Failed to connect to driver")]
    ConnectionError(#[error(source)] io::Error),

    /// Failed to inquire about driver state
    #[error(display = "Failed to inquire about driver state")]
    GetStateError(#[error(source)] io::Error),

    /// Failed to initialize driver
    #[error(display = "Failed to initialize driver")]
    InitializationError(#[error(source)] io::Error),

    /// Failed to register process tree with driver
    #[error(display = "Failed to register process tree with driver")]
    RegisterProcessesError(#[error(source)] io::Error),

    /// Failed to clear configuration in driver
    #[error(display = "Failed to clear configuration in driver")]
    ClearConfigError(#[error(source)] io::Error),

    /// Failed to reset driver state to "started"
    #[error(display = "Failed to reset driver state")]
    ResetError(#[error(source)] io::Error),
}

impl DeviceHandle {
    pub fn new() -> Result<Self, DeviceHandleError> {
        let device = Self::new_handle_only()?;
        device.reinitialize()?;
        Ok(device)
    }

    pub(super) fn new_handle_only() -> Result<Self, DeviceHandleError> {
        log::trace!("Connecting to the driver");

        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .attributes(0)
            .open(DRIVER_SYMBOLIC_NAME)
            .map_err(|e| match e.raw_os_error().map(|raw| raw as u32) {
                Some(ERROR_FILE_NOT_FOUND) => DeviceHandleError::ConnectionFailed,
                Some(ERROR_ACCESS_DENIED) => DeviceHandleError::ConnectionDenied,
                _ => DeviceHandleError::ConnectionError(e),
            })?;
        Ok(Self { handle })
    }

    pub fn reinitialize(&self) -> Result<(), DeviceHandleError> {
        let state = self
            .get_driver_state()
            .map_err(DeviceHandleError::GetStateError)?;
        if state != DriverState::Started {
            log::debug!("Resetting driver state");
            self.reset().map_err(DeviceHandleError::ResetError)?;
        }

        log::debug!("Initializing driver");
        self.initialize()
            .map_err(DeviceHandleError::InitializationError)?;

        log::debug!("Initializing driver process tree");
        self.register_processes()
            .map_err(DeviceHandleError::RegisterProcessesError)
    }

    fn initialize(&self) -> io::Result<()> {
        device_io_control(self, DriverIoctlCode::Initialize as u32, None, 0)?;
        Ok(())
    }

    fn register_processes(&self) -> io::Result<()> {
        let process_tree_buffer = serialize_process_tree(build_process_tree()?)?;
        device_io_control(
            self,
            DriverIoctlCode::RegisterProcesses as u32,
            Some(&process_tree_buffer),
            0,
        )?;
        Ok(())
    }

    pub fn register_ips(
        &self,
        tunnel_ipv4: Option<Ipv4Addr>,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Option<Ipv4Addr>,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> io::Result<()> {
        log::debug!("Register IPs: tunnel IPv4: {:?}, tunnel IPv6 {:?}, internet IPv4: {:?}, internet IPv6: {:?}", tunnel_ipv4, tunnel_ipv6, internet_ipv4, internet_ipv6);
        let mut addresses: SplitTunnelAddresses = unsafe { mem::zeroed() };

        unsafe {
            if let Some(tunnel_ipv4) = tunnel_ipv4 {
                let tunnel_ipv4 = tunnel_ipv4.octets();
                ptr::copy_nonoverlapping(
                    &tunnel_ipv4[0] as *const u8,
                    &mut addresses.tunnel_ipv4 as *mut _ as *mut u8,
                    tunnel_ipv4.len(),
                );
            }

            if let Some(tunnel_ipv6) = tunnel_ipv6 {
                let tunnel_ipv6 = tunnel_ipv6.octets();
                ptr::copy_nonoverlapping(
                    &tunnel_ipv6[0] as *const u8,
                    &mut addresses.tunnel_ipv6 as *mut _ as *mut u8,
                    tunnel_ipv6.len(),
                );
            }

            if let Some(internet_ipv4) = internet_ipv4 {
                let internet_ipv4 = internet_ipv4.octets();
                ptr::copy_nonoverlapping(
                    &internet_ipv4[0] as *const u8,
                    &mut addresses.internet_ipv4 as *mut _ as *mut u8,
                    internet_ipv4.len(),
                );
            }

            if let Some(internet_ipv6) = internet_ipv6 {
                let internet_ipv6 = internet_ipv6.octets();
                ptr::copy_nonoverlapping(
                    &internet_ipv6[0] as *const u8,
                    &mut addresses.internet_ipv6 as *mut _ as *mut u8,
                    internet_ipv6.len(),
                );
            }
        }

        let buffer = as_uninit_byte_slice(&addresses);

        device_io_control(
            self,
            DriverIoctlCode::RegisterIpAddresses as u32,
            Some(buffer),
            0,
        )?;

        Ok(())
    }

    pub fn get_driver_state(&self) -> io::Result<DriverState> {
        let buffer = device_io_control(
            self,
            DriverIoctlCode::GetState as u32,
            None,
            size_of::<u64>() as u32,
        )?
        .unwrap();

        let raw_state: u64 = unsafe { deserialize_buffer(&buffer[0..size_of::<u64>()]) };

        Ok(DriverState::try_from(raw_state)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?)
    }

    pub fn set_config<T: AsRef<OsStr>>(&self, apps: &[T]) -> io::Result<()> {
        let mut device_paths = Vec::with_capacity(apps.len());
        for app in apps.as_ref() {
            match get_device_path(app.as_ref()) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    log::debug!(
                        "{}\nPath: {}",
                        error.display_chain_with_msg("Ignoring path on unmounted volume"),
                        Path::new(app.as_ref()).display()
                    );
                }
                Err(error) => return Err(error),
                Ok(path) => device_paths.push(path),
            }
        }

        if device_paths.is_empty() {
            return self.clear_config();
        }

        log::debug!("Excluded device paths:");
        for path in &device_paths {
            log::debug!("    {}", Path::new(&path).display());
        }

        let config = make_process_config(&device_paths);

        device_io_control(
            self,
            DriverIoctlCode::SetConfiguration as u32,
            Some(&config),
            0,
        )?;

        Ok(())
    }

    pub fn clear_config(&self) -> io::Result<()> {
        device_io_control(self, DriverIoctlCode::ClearConfiguration as u32, None, 0)?;
        Ok(())
    }

    pub(super) fn reset(&self) -> io::Result<()> {
        device_io_control(self, DriverIoctlCode::Reset as u32, None, 0)?;
        Ok(())
    }
}

impl AsRawHandle for DeviceHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct SplitTunnelAddresses {
    tunnel_ipv4: IN_ADDR,
    internet_ipv4: IN_ADDR,
    tunnel_ipv6: IN6_ADDR,
    internet_ipv6: IN6_ADDR,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ConfigurationHeader {
    // Number of entries immediately following the header.
    num_entries: usize,
    // Total byte length: header + entries + string buffer.
    total_length: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ConfigurationEntry {
    // Offset into buffer region that follows all entries.
    // The image name uses the physical path.
    name_offset: usize,
    // Byte length for non-null terminated wide char string.
    name_length: u16,
}

/// Create a buffer containing a `ConfigurationHeader` and number of `ConfigurationEntry`s,
/// followed by the same number of paths to those entries.
fn make_process_config<T: AsRef<OsStr>>(apps: &[T]) -> Vec<MaybeUninit<u8>> {
    let apps: Vec<Vec<u16>> = apps
        .iter()
        .map(|app| app.as_ref().encode_wide().collect())
        .collect();

    let total_string_size: usize = apps.iter().map(|app| size_of::<u16>() * app.len()).sum();

    let total_buffer_size = size_of::<ConfigurationHeader>()
        + size_of::<ConfigurationEntry>() * apps.len()
        + total_string_size;

    let mut buffer = Vec::<MaybeUninit<u8>>::new();
    buffer.resize(total_buffer_size, MaybeUninit::new(0));

    let (header, tail) = buffer.split_at_mut(size_of::<ConfigurationHeader>());

    // Serialize configuration header
    let header_struct = ConfigurationHeader {
        num_entries: apps.len(),
        total_length: total_buffer_size,
    };
    header.copy_from_slice(as_uninit_byte_slice(&header_struct));

    // Serialize configuration entries and strings
    let (entries, string_data) = tail.split_at_mut(apps.len() * size_of::<ConfigurationEntry>());
    let mut string_offset = 0;

    for (i, app) in apps.iter().enumerate() {
        write_string_to_buffer(string_data, string_offset, &app);

        let app_bytelen = size_of::<u16>() * app.len();
        let entry = ConfigurationEntry {
            name_offset: string_offset,
            name_length: app_bytelen as u16,
        };
        let entry_offset = size_of::<ConfigurationEntry>() * i;

        entries[entry_offset..entry_offset + size_of::<ConfigurationEntry>()]
            .copy_from_slice(as_uninit_byte_slice(&entry));

        string_offset += app_bytelen;
    }

    buffer
}

#[derive(Debug)]
struct ProcessInfo {
    pid: u32,
    parent_pid: u32,
    creation_time: u64,
    device_path: Vec<u16>,
}

/// List process identifiers, their parents, and their device paths.
fn build_process_tree() -> io::Result<Vec<ProcessInfo>> {
    let mut process_info = HashMap::new();

    let snap = ProcessSnapshot::new(TH32CS_SNAPPROCESS, 0)?;
    for entry in snap.entries() {
        let entry = entry?;

        let process = match open_process(ProcessAccess::QueryLimitedInformation, false, entry.pid) {
            Ok(handle) => Ok(handle),
            Err(error) => {
                // Skip process objects that cannot be opened
                match error.kind() {
                    // System process
                    io::ErrorKind::PermissionDenied => continue,
                    // System idle or csrss process
                    io::ErrorKind::InvalidInput => continue,
                    io::ErrorKind::Other => {
                        // Old rust lib maps INVALID_PARAMETER to "Other"
                        if error.raw_os_error() == Some(ERROR_INVALID_PARAMETER as i32) {
                            continue;
                        }
                        Err(error)
                    }
                    _ => Err(error),
                }
            }
        }?;

        // TODO: Skip objects whose paths or timestamps cannot be obtained?

        process_info.insert(
            entry.pid,
            RefCell::new(ProcessInfo {
                pid: entry.pid,
                parent_pid: entry.parent_pid,
                creation_time: get_process_creation_time(process.get_raw()).unwrap_or(0),
                device_path: get_process_device_path(process.get_raw())
                    .unwrap_or(OsString::from(""))
                    .encode_wide()
                    .collect(),
            }),
        );
    }

    // Handle pid recycling
    // If the "parent" is younger than the process itself, it is not our parent.
    for info in process_info.values() {
        let mut info = info.borrow_mut();
        let parent_pid = info.parent_pid;
        if parent_pid == 0 {
            continue;
        }
        if let Some(parent_info) = process_info.get(&parent_pid) {
            if parent_info.borrow_mut().creation_time > info.creation_time {
                info.parent_pid = 0;
            }
        }
    }

    Ok(process_info
        .into_iter()
        .map(|(_, info)| info.into_inner())
        .collect())
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ProcessRegistryHeader {
    // Number of entries immediately following the header.
    num_entries: usize,
    // Total byte length: header + entries + string buffer.
    total_length: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ProcessRegistryEntry {
    pid: RawHandle,
    parent_pid: RawHandle,
    // Image name offset (following the last entry).
    image_name_offset: usize,
    // Image name length.
    image_name_size: u16,
}

fn serialize_process_tree(processes: Vec<ProcessInfo>) -> Result<Vec<MaybeUninit<u8>>, io::Error> {
    // Construct a buffer:
    //  ProcessRegistryHeader
    //  ProcessRegistryEntry..
    //  Image names..

    let total_string_size: usize = processes
        .iter()
        .map(|info| size_of::<u16>() * info.device_path.len())
        .sum();
    let total_buffer_size = size_of::<ProcessRegistryHeader>()
        + size_of::<ProcessRegistryEntry>() * processes.len()
        + total_string_size;

    let mut buffer = Vec::new();
    buffer.resize(total_buffer_size, MaybeUninit::new(0u8));

    let (header, tail) = buffer.split_at_mut(size_of::<ProcessRegistryHeader>());
    let header_struct = ProcessRegistryHeader {
        num_entries: processes.len(),
        total_length: total_buffer_size,
    };
    header.copy_from_slice(as_uninit_byte_slice(&header_struct));

    let (entries, string_data) =
        tail.split_at_mut(size_of::<ProcessRegistryEntry>() * processes.len());

    let mut string_offset = 0;

    for (i, entry) in processes.into_iter().enumerate() {
        let mut out_entry = ProcessRegistryEntry {
            pid: entry.pid as usize as RawHandle,
            parent_pid: entry.parent_pid as usize as RawHandle,
            image_name_size: 0,
            image_name_offset: 0,
        };

        if !entry.device_path.is_empty() {
            write_string_to_buffer(string_data, string_offset, &entry.device_path);

            out_entry.image_name_size = (entry.device_path.len() * size_of::<u16>()) as u16;
            out_entry.image_name_offset = string_offset;

            string_offset += size_of::<u16>() * entry.device_path.len();
        }

        let entry_offset = size_of::<ProcessRegistryEntry>() * i;
        entries[entry_offset..entry_offset + size_of::<ProcessRegistryEntry>()]
            .copy_from_slice(as_uninit_byte_slice(&out_entry));
    }

    Ok(buffer)
}

#[repr(C)]
struct EventHeader {
    event_id: EventId,
    event_size: usize,
    event_data: [u8; 0],
}

#[repr(C)]
struct SplittingEventHeader {
    process_id: usize,
    reason: u32,
    image_name_length: u16,
    image_name_data: [u16; 0],
}

#[repr(C)]
struct SplittingErrorEventHeader {
    process_id: usize,
    image_name_length: u16,
    image_name_data: [u16; 0],
}

#[repr(C)]
struct ErrorMessageEventHeader {
    status: NTSTATUS,
    error_message_length: u16,
    error_message_data: [u16; 0],
}

/// Parses an event returned by the ST driver.
///
/// # Panics
///
/// This may panic if `buffer` contains invalid data.
pub fn parse_event_buffer(buffer: &[u8]) -> Result<(EventId, EventBody), UnknownEventId> {
    // SAFETY: This panics if `buffer` is too small.
    let raw_event_id: u32 = unsafe { deserialize_buffer(&buffer[0..mem::size_of::<u32>()]) };
    let _event_id = EventId::try_from(raw_event_id)?;

    // SAFETY: The event id is known to be valid.
    let event_header: EventHeader =
        unsafe { deserialize_buffer(&buffer[0..offset_of!(EventHeader, event_data)]) };

    let (_, buffer) = buffer.split_at(offset_of!(EventHeader, event_data));

    match event_header.event_id {
        EventId::StartSplittingProcess | EventId::StopSplittingProcess => {
            // SAFETY: This will panic if the buffer is too small to contain the message.
            let event: SplittingEventHeader = unsafe {
                deserialize_buffer(&buffer[0..offset_of!(SplittingEventHeader, image_name_data)])
            };
            let string_byte_offset = offset_of!(SplittingEventHeader, image_name_data);
            let image = buffer_to_osstring(
                &buffer
                    [string_byte_offset..(string_byte_offset + event.image_name_length as usize)],
            );

            Ok((
                event_header.event_id,
                EventBody::SplittingEvent {
                    process_id: event.process_id,
                    reason: SplittingChangeReason::from_bits(event.reason).unwrap_or_else(|| {
                        log::error!("Dropping unknown bits from splitting change reason. Original reason: {:b}", event.reason);
                        SplittingChangeReason::from_bits_truncate(event.reason)
                    }),
                    image,
                },
            ))
        }
        EventId::ErrorStartSplittingProcess | EventId::ErrorStopSplittingProcess => {
            // SAFETY: This will panic if the buffer is too small to contain the message.
            let event: SplittingErrorEventHeader = unsafe {
                deserialize_buffer(
                    &buffer[0..offset_of!(SplittingErrorEventHeader, image_name_data)],
                )
            };
            let string_byte_offset = offset_of!(SplittingErrorEventHeader, image_name_data);
            let image = buffer_to_osstring(
                &buffer
                    [string_byte_offset..(string_byte_offset + event.image_name_length as usize)],
            );

            Ok((
                event_header.event_id,
                EventBody::SplittingError {
                    process_id: event.process_id,
                    image,
                },
            ))
        }
        EventId::ErrorMessage => {
            // SAFETY: This will panic if the buffer is too small to contain the message.
            let event: ErrorMessageEventHeader = unsafe {
                deserialize_buffer(
                    &buffer[0..offset_of!(ErrorMessageEventHeader, error_message_data)],
                )
            };
            let string_byte_offset = offset_of!(ErrorMessageEventHeader, error_message_data);
            let message = buffer_to_osstring(
                &buffer[string_byte_offset
                    ..(string_byte_offset + event.error_message_length as usize)],
            );

            Ok((
                event_header.event_id,
                EventBody::ErrorMessage {
                    status: event.status,
                    message,
                },
            ))
        }
    }
}

/// Send an IOCTL code to the given device handle, and wait for the result.
///
/// `input` specifies an optional buffer for sending data.
///
/// Upon success, a buffer containing at most `output_size` bytes is returned,
/// or `None` if no bytes were read.
pub fn device_io_control(
    device: &DeviceHandle,
    ioctl_code: u32,
    input: Option<&[MaybeUninit<u8>]>,
    output_size: u32,
) -> Result<Option<Vec<u8>>, io::Error> {
    let mut overlapped = Overlapped::new(Some(Event::new(true, false)?))?;

    let mut buffer = vec![];
    let out_buffer = if output_size > 0 {
        buffer.resize(
            usize::try_from(output_size).expect("u32 must be no larger than usize"),
            0u8,
        );
        Some(&mut buffer[..])
    } else {
        None
    };

    let bytes_read =
        device_io_control_buffer(device, ioctl_code, input, out_buffer, &mut overlapped)?;
    if bytes_read > 0 {
        buffer.truncate(usize::try_from(bytes_read).expect("u32 must be no larger than usize"));
        return Ok(Some(buffer));
    }
    Ok(None)
}

/// Send an IOCTL code to the given device handle, and wait for the result.
///
/// `input` specifies an optional buffer for sending data.
///
/// Upon success, `output` buffer will contain at most `output.len()` bytes of data,
/// and the function returns the number of bytes read.
///
/// # Panics
///
/// This function will panic if `overlapped` does not contain an event.
pub fn device_io_control_buffer(
    device: &DeviceHandle,
    ioctl_code: u32,
    input: Option<&[MaybeUninit<u8>]>,
    output: Option<&mut [u8]>,
    overlapped: &mut Overlapped,
) -> Result<u32, io::Error> {
    let output_len = output.as_ref().map(|output| output.len()).unwrap_or(0);
    let output_len = u32::try_from(output_len).map_err(|_error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "the output buffer is too large",
        )
    })?;
    let out_ptr = match output {
        Some(output) => output as *mut _ as *mut _,
        None => ptr::null_mut(),
    };
    // SAFETY: `out_ptr` will be valid until the result has been obtained.
    unsafe {
        device_io_control_buffer_async(
            device,
            ioctl_code,
            input,
            out_ptr,
            output_len,
            overlapped.as_mut_ptr(),
        )?;
    }
    get_overlapped_result(device, overlapped)
}

/// Send an IOCTL code to the given device handle.
///
/// `input` specifies an optional buffer for sending data.
/// `output_ptr` specifies an optional buffer for receiving data.
///
/// Obtain the result using [get_overlapped_result].
///
/// # Safety
///
/// * `output_ptr` must either be null or a valid buffer of `output_len` bytes. It must remain valid
///   until the overlapped operation has completed.
pub unsafe fn device_io_control_buffer_async(
    device: &DeviceHandle,
    ioctl_code: u32,
    input: Option<&[MaybeUninit<u8>]>,
    output_ptr: *mut u8,
    output_len: u32,
    overlapped: *mut OVERLAPPED,
) -> Result<(), io::Error> {
    let input_ptr = match input {
        Some(input) => input.as_ptr() as *mut _,
        None => ptr::null_mut(),
    };
    let input_len = input.map(|input| input.len()).unwrap_or(0);

    let result = DeviceIoControl(
        device.as_raw_handle() as HANDLE,
        ioctl_code,
        input_ptr,
        u32::try_from(input_len).map_err(|_error| {
            io::Error::new(io::ErrorKind::InvalidInput, "the input buffer is too large")
        })?,
        output_ptr as *mut _,
        output_len,
        ptr::null_mut(),
        overlapped,
    );

    if result != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Expected pending operation",
        ));
    }

    let last_error = io::Error::last_os_error();
    if last_error.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
        return Err(last_error);
    }

    Ok(())
}

/// Retrieves the result of an overlapped operation. On success, this returns
/// the number of bytes transferred. For device I/O, this is the number of bytes
/// written to the output buffer.
///
/// # Panics
///
/// This function will panic if `overlapped` does not contain an event.
pub fn get_overlapped_result(
    device: &DeviceHandle,
    overlapped: &mut Overlapped,
) -> io::Result<u32> {
    let event = overlapped.get_event().unwrap();

    // SAFETY: This is a valid event object.
    unsafe { wait_for_single_object(event.as_handle(), None) }?;

    // SAFETY: The handle and overlapped object are valid.
    let mut returned_bytes = 0u32;
    let result = unsafe {
        GetOverlappedResult(
            device.as_raw_handle() as HANDLE,
            overlapped.as_mut_ptr(),
            &mut returned_bytes,
            0,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(returned_bytes)
}

/// Waits for an object to be signaled, or until a timeout interval has elapsed.
///
/// # Safety
///
/// * `object` must be a valid object that can be signaled, such as an event object.
pub unsafe fn wait_for_single_object(object: HANDLE, timeout: Option<Duration>) -> io::Result<()> {
    let timeout = match timeout {
        Some(timeout) => u32::try_from(timeout.as_millis()).map_err(|_error| {
            io::Error::new(io::ErrorKind::InvalidInput, "the duration is too long")
        })?,
        None => INFINITE,
    };
    let result = WaitForSingleObject(object, timeout);
    match result {
        WAIT_OBJECT_0 => Ok(()),
        WAIT_FAILED => Err(io::Error::last_os_error()),
        WAIT_ABANDONED => Err(io::Error::new(io::ErrorKind::Other, "abandoned mutex")),
        error => Err(io::Error::from_raw_os_error(error as i32)),
    }
}

/// Waits for one or several objects to be signaled. On success, this returns a pointer to an
/// object in `objects` that was signaled.
///
/// # Safety
///
/// * `objects` must be a slice of valid objects that can be signaled, such as event objects.
pub unsafe fn wait_for_multiple_objects(objects: &[HANDLE], wait_all: bool) -> io::Result<HANDLE> {
    let objects_len = u32::try_from(objects.len())
        .map_err(|_error| io::Error::new(io::ErrorKind::InvalidInput, "too many objects"))?;
    let result = WaitForMultipleObjects(
        objects_len,
        objects.as_ptr(),
        if wait_all { 1 } else { 0 },
        INFINITE,
    );
    let signaled_index = if result >= WAIT_OBJECT_0 && result < WAIT_OBJECT_0 + objects_len {
        result - WAIT_OBJECT_0
    } else if result >= WAIT_ABANDONED_0 && result < WAIT_ABANDONED_0 + objects_len {
        return Err(io::Error::new(io::ErrorKind::Other, "abandoned mutex"));
    } else {
        return Err(io::Error::last_os_error());
    };
    Ok(objects[usize::try_from(signaled_index).expect("usize must be larger than u32")])
}

/// Reads the value from `buffer`, zeroing any remaining bytes.
///
/// # Safety
///
/// The caller must ensure that `T` is initialized by the byte buffer.
///
/// # Panics
///
/// This panics if `buffer` is larger than `T`.
unsafe fn deserialize_buffer<T>(buffer: &[u8]) -> T {
    assert!(buffer.len() <= mem::size_of::<T>());

    let mut instance = MaybeUninit::zeroed();
    ptr::copy_nonoverlapping(
        buffer.as_ptr(),
        instance.as_mut_ptr() as *mut u8,
        buffer.len(),
    );
    instance.assume_init()
}

fn buffer_to_osstring(buffer: &[u8]) -> OsString {
    let mut out_buf = Vec::new();
    out_buf.resize((buffer.len() + 1) / mem::size_of::<u16>(), 0u16);

    // SAFETY: `out_buf` contains enough bytes to store all of `buffer`.
    unsafe {
        ptr::copy_nonoverlapping(
            buffer as *const _ as *const u16,
            out_buf.as_mut_ptr(),
            out_buf.len(),
        )
    };
    OsStringExt::from_wide(&out_buf)
}

/// Inserts a string into `buffer` at a given `byte_offset`.
///
/// # Panics
///
/// This panics if either `byte_offset` or `byte_offset + 2 * string.len() - 1` is
/// an out of bounds index for `buffer`.
fn write_string_to_buffer(buffer: &mut [MaybeUninit<u8>], byte_offset: usize, string: &[u16]) {
    for (i, byte) in string
        .iter()
        .flat_map(|word| word.to_ne_bytes().into_iter())
        .enumerate()
    {
        buffer[byte_offset + i] = MaybeUninit::new(byte);
    }
}

/// Casts a struct to a slice of possibly uninitialized bytes.
pub fn as_uninit_byte_slice<T: Copy + Sized>(value: &T) -> &[mem::MaybeUninit<u8>] {
    unsafe { std::slice::from_raw_parts(value as *const _ as *const _, mem::size_of::<T>()) }
}
