use super::windows::{
    get_final_path_name, get_process_creation_time, get_process_device_path, open_process,
    ProcessAccess, ProcessSnapshot,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io,
    mem::{self, size_of},
    net::{Ipv4Addr, Ipv6Addr},
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, RawHandle},
    },
    ptr,
};
use winapi::{
    shared::{in6addr::IN6_ADDR, inaddr::IN_ADDR},
    um::{
        ioapiset::DeviceIoControl,
        tlhelp32::TH32CS_SNAPPROCESS,
        winioctl::{FILE_ANY_ACCESS, METHOD_BUFFERED, METHOD_NEITHER},
    },
};

const DRIVER_SYMBOLIC_NAME: &str = "\\\\.\\MULLVADSPLITTUNNEL";
const ST_DEVICE_TYPE: u32 = 0x8000;

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    device_type << 16 | access << 14 | function << 2 | method
}

#[repr(u32)]
#[allow(dead_code)]
enum DriverIoctlCode {
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

pub struct DeviceHandle {
    handle: fs::File,
}

impl DeviceHandle {
    pub fn new() -> io::Result<Self> {
        // Connect to the driver
        log::trace!("Connecting to the driver");
        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .custom_flags(0)
            .attributes(0)
            .open(DRIVER_SYMBOLIC_NAME)?;

        let device = Self { handle };

        // Initialize the driver
        let state = device.get_driver_state()?;
        if state == DriverState::Started {
            log::trace!("Initializing driver");
            device.initialize()?;
        }

        // Initialize process tree
        let state = device.get_driver_state()?;
        if state == DriverState::Initialized {
            log::trace!("Registering processes");
            device.register_processes()?;
        }

        Ok(device)
    }

    fn initialize(&self) -> io::Result<()> {
        device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::Initialize as u32,
            None,
            0,
        )?;
        Ok(())
    }

    fn register_processes(&self) -> io::Result<()> {
        let process_tree_buffer = serialize_process_tree(build_process_tree()?)?;
        device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::RegisterProcesses as u32,
            Some(&process_tree_buffer),
            0,
        )?;
        Ok(())
    }

    pub fn register_ips(
        &self,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> io::Result<()> {
        let mut addresses: SplitTunnelAddresses = unsafe { mem::zeroed() };

        unsafe {
            let tunnel_ipv4 = tunnel_ipv4.octets();
            ptr::copy_nonoverlapping(
                &tunnel_ipv4[0] as *const u8,
                &mut addresses.tunnel_ipv4 as *mut _ as *mut u8,
                tunnel_ipv4.len(),
            );

            if let Some(tunnel_ipv6) = tunnel_ipv6 {
                let tunnel_ipv6 = tunnel_ipv6.octets();
                ptr::copy_nonoverlapping(
                    &tunnel_ipv6[0] as *const u8,
                    &mut addresses.tunnel_ipv6 as *mut _ as *mut u8,
                    tunnel_ipv6.len(),
                );
            }

            let internet_ipv4 = internet_ipv4.octets();
            ptr::copy_nonoverlapping(
                &internet_ipv4[0] as *const u8,
                &mut addresses.internet_ipv4 as *mut _ as *mut u8,
                internet_ipv4.len(),
            );

            if let Some(internet_ipv6) = internet_ipv6 {
                let internet_ipv6 = internet_ipv6.octets();
                ptr::copy_nonoverlapping(
                    &internet_ipv6[0] as *const u8,
                    &mut addresses.internet_ipv6 as *mut _ as *mut u8,
                    internet_ipv6.len(),
                );
            }
        }

        let buffer = &addresses as *const _ as *const u8;
        let buffer =
            unsafe { std::slice::from_raw_parts(buffer, size_of::<SplitTunnelAddresses>()) };

        device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::RegisterIpAddresses as u32,
            Some(buffer),
            0,
        )?;

        Ok(())
    }

    pub fn get_driver_state(&self) -> io::Result<DriverState> {
        let buffer = device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::GetState as u32,
            None,
            size_of::<u64>() as u32,
        )?
        .unwrap();

        Ok(unsafe { deserialize_buffer(&buffer) })
    }

    pub fn set_config<T: AsRef<OsStr>>(&self, apps: &[T]) -> io::Result<()> {
        let mut device_paths = Vec::with_capacity(apps.len());
        for app in apps.as_ref() {
            device_paths.push(get_final_path_name(app)?);
        }

        log::debug!("Excluded device paths:");
        for path in &device_paths {
            log::debug!("    {:?}", path);
        }

        let config = make_process_config(&device_paths);

        device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::SetConfiguration as u32,
            Some(&config),
            0,
        )?;

        Ok(())
    }

    pub fn clear_config(&self) -> io::Result<()> {
        device_io_control(
            self.handle.as_raw_handle(),
            DriverIoctlCode::ClearConfiguration as u32,
            None,
            0,
        )?;

        Ok(())
    }
}

#[repr(C)]
struct SplitTunnelAddresses {
    tunnel_ipv4: IN_ADDR,
    internet_ipv4: IN_ADDR,
    tunnel_ipv6: IN6_ADDR,
    internet_ipv6: IN6_ADDR,
}

#[repr(C)]
struct ConfigurationHeader {
    // Number of entries immediately following the header.
    num_entries: usize,
    // Total byte length: header + entries + string buffer.
    total_length: usize,
}

#[repr(C)]
struct ConfigurationEntry {
    // Offset into buffer region that follows all entries.
    // The image name uses the physical path.
    name_offset: usize,
    // Byte length for non-null terminated wide char string.
    name_length: u16,
}

/// Create a buffer containing a `ConfigurationHeader` and number of `ConfigurationEntry`s,
/// followed by the same number of paths to those entries.
fn make_process_config<T: AsRef<OsStr>>(apps: &[T]) -> Vec<u8> {
    let apps: Vec<Vec<u16>> = apps
        .iter()
        .map(|app| app.as_ref().encode_wide().collect())
        .collect();

    let total_string_size: usize = apps.iter().map(|app| size_of::<u16>() * app.len()).sum();

    let total_buffer_size = size_of::<ConfigurationHeader>()
        + size_of::<ConfigurationEntry>() * apps.len()
        + total_string_size;

    let mut buffer = Vec::<u8>::new();
    buffer.resize(total_buffer_size, 0);

    let (header, tail) = buffer.split_at_mut(size_of::<ConfigurationHeader>());

    // Serialize configuration header
    let header_struct = ConfigurationHeader {
        num_entries: apps.len(),
        total_length: total_buffer_size,
    };
    header.copy_from_slice(unsafe { as_u8_slice(&header_struct) });

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
            .copy_from_slice(unsafe { as_u8_slice(&entry) });

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

#[repr(C)]
struct ProcessRegistryHeader {
    // Number of entries immediately following the header.
    num_entries: usize,
    // Total byte length: header + entries + string buffer.
    total_length: usize,
}

#[repr(C)]
struct ProcessRegistryEntry {
    pid: RawHandle,
    parent_pid: RawHandle,
    // Image name offset (following the last entry).
    image_name_offset: usize,
    // Image name length.
    image_name_size: u16,
}

fn serialize_process_tree(processes: Vec<ProcessInfo>) -> Result<Vec<u8>, io::Error> {
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

    let mut buffer = Vec::<u8>::new();
    buffer.resize(total_buffer_size, 0);

    let (header, tail) = buffer.split_at_mut(size_of::<ProcessRegistryHeader>());
    let header_struct = ProcessRegistryHeader {
        num_entries: processes.len(),
        total_length: total_buffer_size,
    };
    header.copy_from_slice(unsafe { as_u8_slice(&header_struct) });

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
            .copy_from_slice(unsafe { as_u8_slice(&out_entry) });
    }

    Ok(buffer)
}

/// Send an IOCTL code to the given device handle.
/// `input` specifies an optional buffer to send.
/// Upon success, a buffer of size `output_size` is returned, or None if `output_size` is 0.
pub fn device_io_control(
    device: RawHandle,
    ioctl_code: u32,
    input: Option<&[u8]>,
    output_size: u32,
) -> Result<Option<Vec<u8>>, io::Error> {
    let input_ptr = match input {
        Some(input) => input as *const _ as *mut _,
        None => ptr::null_mut(),
    };
    let input_len = input.map(|input| input.len()).unwrap_or(0);

    let mut out_buffer = if output_size > 0 {
        Some(Vec::with_capacity(output_size as usize))
    } else {
        None
    };

    let out_ptr = match out_buffer {
        Some(ref mut out_buffer) => out_buffer.as_mut_ptr() as *mut _,
        None => ptr::null_mut(),
    };

    let mut returned_bytes = 0u32;

    let result = unsafe {
        DeviceIoControl(
            device as *mut _,
            ioctl_code,
            input_ptr,
            input_len as u32,
            out_ptr,
            output_size,
            &mut returned_bytes as *mut _,
            ptr::null_mut(), // TODO
        )
    };

    if let Some(ref mut out_buffer) = out_buffer {
        unsafe { out_buffer.set_len(returned_bytes as usize) };
    }

    if result != 0 {
        Ok(out_buffer)
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Creates a new instance of an arbitrary type from a byte buffer.
pub unsafe fn deserialize_buffer<T: Sized>(buffer: &Vec<u8>) -> T {
    let mut instance: T = mem::zeroed();
    ptr::copy_nonoverlapping(buffer.as_ptr() as *const T, &mut instance as *mut _, 1);
    instance
}

fn write_string_to_buffer(buffer: &mut [u8], byte_offset: usize, string: &[u16]) {
    for (i, byte) in string
        .iter()
        .flat_map(|word| std::array::IntoIter::new(word.to_ne_bytes()))
        .enumerate()
    {
        buffer[byte_offset + i] = byte;
    }
}

unsafe fn as_u8_slice<T: Sized>(object: &T) -> &[u8] {
    std::slice::from_raw_parts(object as *const _ as *const _, size_of::<T>())
}
