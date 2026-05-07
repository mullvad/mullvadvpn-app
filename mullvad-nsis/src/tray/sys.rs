use std::ffi::OsString;
use std::io;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;

use widestring::U16CString;
use windows_registry::{CURRENT_USER, Key, Type};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes};

use std::os::windows::io::{AsHandle, AsRawHandle, FromRawHandle, OwnedHandle};
use windows_sys::Win32::Foundation::{FALSE, HANDLE, MAX_PATH, SYSTEMTIME};
use windows_sys::Win32::Security::{
    DuplicateTokenEx, SECURITY_IMPERSONATION_LEVEL, SecurityImpersonation, TOKEN_ALL_ACCESS,
    TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_QUERY, TOKEN_TYPE, TokenPrimary,
};
use windows_sys::Win32::System::ProcessStatus::EnumProcesses;
use windows_sys::Win32::System::SystemInformation::GetSystemTime;
use windows_sys::Win32::System::Threading::{
    CreateProcessWithTokenW, INFINITE, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, QueryFullProcessImageNameW, STARTUPINFOW,
    TerminateProcess, WaitForSingleObject,
};
use windows_sys::Win32::System::Time::SystemTimeToFileTime;
use windows_sys::Win32::UI::Shell::FOLDERID_Windows;
use windows_sys::Win32::UI::Shell::KF_FLAG_DEFAULT;

use mullvad_paths::{PRODUCT_NAME, windows::get_known_folder_path};

/// Template for a new Mullvad VPN tray record in `IconStreams`.
const MULLVAD_TRAY_RECORD_TEMPLATE: &[u8] = include_bytes!("mullvad_tray_record.bin");

/// ICON_STREAMS_HEADER
///
/// This is the header of the `IconStreams` binary blob in registry value.
#[repr(C, packed)]
#[derive(Clone, Copy, IntoBytes, TryFromBytes, KnownLayout, Immutable)]
struct IconStreamsHeader {
    header_size: IconStreamsHeaderSize,
    u1: u32,
    u2: u16,
    u3: u16,
    number_records: u32,
    offset_first_record: IconStreamsHeaderSize,
}

#[derive(Clone, Copy, IntoBytes, TryFromBytes, KnownLayout, Immutable)]
#[repr(u32)]
enum IconStreamsHeaderSize {
    Size = 20,
}

const _: () = assert!(
    IconStreamsHeaderSize::Size as usize == mem::size_of::<IconStreamsHeader>(),
    "unexpected header size for IconStreamsHeader"
);

/// Tray icon visibility constants
const SHOW_ICON_AND_NOTIFICATIONS: u32 = 2;

/// ICON_STREAMS_RECORD
///
/// This is an entry in the `IconStreams` binary blob registry value.
#[repr(C, packed)]
#[derive(Clone, Copy, IntoBytes, FromBytes, KnownLayout, Immutable)]
struct IconStreamsRecord {
    application_path: [u16; MAX_PATH as usize],
    /// ID used to identify an icon
    u1: u32,
    // 0
    u2: u32,
    visibility: u32,
    year_created: u16,
    month_created: u16,
    last_tooltip: [u16; MAX_PATH as usize],
    // 0
    u6: u32,
    // 0 or 1, don't know why
    u7: u32,
    // ID of cached icon, or -1
    imagelist_id: u32,
    guid: [u8; 16],
    // 0
    u8_: u32,
    // 0
    u9: u32,
    // 0
    u10: u32,
    /// Discrete event 1 UTC
    time1: FILETIME,
    /// Discrete event 2 UTC, or 0
    time2: FILETIME,
    // 0
    u11: u32,
    dummy_union: DummyUnion,
    /// Ordering within group
    ordinal: u32,
}

#[repr(C, packed)]
#[derive(IntoBytes, TryFromBytes, KnownLayout, Immutable)]
struct IconStreamsBlob {
    header: IconStreamsHeader,
    records: [IconStreamsRecord],
}

#[repr(C)]
#[derive(Clone, Copy, IntoBytes, FromBytes, KnownLayout, Immutable)]
#[expect(clippy::upper_case_acronyms)]
pub struct FILETIME {
    pub low_date_time: u32,
    pub high_date_time: u32,
}

const _: () = {
    assert!(
        mem::size_of::<windows_sys::Win32::Foundation::FILETIME>() == mem::size_of::<FILETIME>()
    );
    assert!(
        mem::align_of::<windows_sys::Win32::Foundation::FILETIME>() == mem::align_of::<FILETIME>()
    );
};

/// ICON_STREAMS_RECORD union details variant
#[repr(C, packed)]
#[derive(Clone, Copy, IntoBytes, FromBytes, KnownLayout, Immutable)]
struct DetailsVariant {
    application_name: [u16; 257],
    padding: [u8; 6],
}

/// ICON_STREAMS_RECORD union extended_details variant
#[repr(C, packed)]
#[derive(Clone, Copy, IntoBytes, FromBytes, KnownLayout, Immutable)]
struct ExtendedDetailsVariant {
    // 0x200d0000
    u12: u32,
    // 0xb0fe
    u13: u16,
    application_name: [u16; 257],
}

const _: () = assert!(mem::size_of::<DetailsVariant>() == mem::size_of::<ExtendedDetailsVariant>());

/// ICON_STREAMS_RECORD union
#[repr(C, packed)]
#[derive(Clone, Copy, IntoBytes, FromBytes, KnownLayout, Immutable)]
union DummyUnion {
    details: DetailsVariant,
    extended_details: ExtendedDetailsVariant,
}

const RECORD_SIZE: usize = mem::size_of::<IconStreamsRecord>();
const _: () = assert!(RECORD_SIZE == 1640);

/// Parsed contents of the `IconStreams` registry value, plus an open
/// handle to the registry key it was loaded from.
struct IconStreams {
    key: Key,
    header: IconStreamsHeader,
    records: Vec<IconStreamsRecord>,
}

impl IconStreams {
    /// Registry key holding the tray icon data.
    const KEY_NAME: &str = "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\TrayNotify";
    /// Registry value within `KEY_NAME` holding the IconStreams blob.
    const VALUE_NAME: &str = "IconStreams";

    /// Open the IconStreams registry key and parse the current blob.
    fn read() -> io::Result<Self> {
        let key = CURRENT_USER.options().read().write().open(Self::KEY_NAME)?;
        let value = key.get_value(Self::VALUE_NAME)?;
        if value.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "IconStreams registry value is empty",
            ));
        }
        let (header, records) = Self::parse_blob(&value)?;
        Ok(Self {
            key,
            header,
            records,
        })
    }

    /// Pack and write the IconStreams blob back to its registry key.
    fn write(&self) -> io::Result<()> {
        self.key
            .set_bytes(Self::VALUE_NAME, Type::Bytes, &self.pack_blob())?;
        Ok(())
    }

    /// Find the record whose decoded ApplicationPath contains `substring`.
    fn find_record(&mut self, substring: &str) -> Option<&mut IconStreamsRecord> {
        self.records.iter_mut().find(|r| {
            // we copy the path here because it is potentially unaligned
            let path = decode_application_path(&{ r.application_path });
            path.contains(substring)
        })
    }

    /// Get the highest ordinal among visible records, plus one.
    /// Returns `None` if no visible records exist.
    fn next_free_ordinal(&self) -> Option<u32> {
        self.records
            .iter()
            .filter(|r| r.visibility == SHOW_ICON_AND_NOTIFICATIONS)
            .map(|r| r.ordinal)
            .max()
            .map(|highest| highest + 1)
    }

    /// Inject a new Mullvad tray record using the embedded template.
    fn inject_mullvad_record(&mut self) -> io::Result<()> {
        const _: () = assert!(
            MULLVAD_TRAY_RECORD_TEMPLATE.len() == RECORD_SIZE,
            "file must match record struct layout"
        );

        let mut new_record =
            IconStreamsRecord::read_from_bytes(MULLVAD_TRAY_RECORD_TEMPLATE).unwrap();

        let now = current_filetime();
        let ordinal = self.next_free_ordinal().unwrap_or(0);

        // Get current system time for year/month fields
        let st = get_system_time();

        new_record.visibility = SHOW_ICON_AND_NOTIFICATIONS;
        new_record.year_created = st.wYear;
        new_record.month_created = st.wMonth;
        new_record.u7 = 0;
        new_record.imagelist_id = 0xFFFFFFFF;
        new_record.time1 = now;
        new_record.time2 = FILETIME::new_zeroed();
        new_record.ordinal = ordinal;

        self.records.push(new_record);
        Ok(())
    }

    /// Parse the binary `IconStreams` registry blob into header + records.
    fn parse_blob(blob: &[u8]) -> io::Result<(IconStreamsHeader, Vec<IconStreamsRecord>)> {
        let IconStreamsBlob { header, records } = IconStreamsBlob::try_ref_from_bytes(blob)
            .map_err(|_err| io::Error::other("Invalid IconStreams blob"))?;

        if usize::try_from(header.number_records).unwrap() != records.len() {
            let num_records = header.number_records;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid record count in IconStreamsBlob: expected {}, got {}",
                    num_records,
                    records.len()
                ),
            ));
        }

        Ok((*header, records.to_vec()))
    }

    /// Serialize header + records back into a binary blob.
    fn pack_blob(&self) -> Vec<u8> {
        let mut packed_header = self.header;
        packed_header.number_records = self.records.len() as u32;

        let mut blob = packed_header.as_bytes().to_vec();
        blob.extend(self.records.as_bytes());
        blob
    }
}

/// Get the current system time.
fn get_system_time() -> SYSTEMTIME {
    let mut st = SYSTEMTIME::default();
    // SAFETY: `&mut st` points to a stack-local SYSTEMTIME the API fills in.
    unsafe { GetSystemTime(&raw mut st) };
    st
}

/// Get the current system time as a FILETIME.
fn current_filetime() -> FILETIME {
    let st = get_system_time();
    let mut ft = FILETIME::new_zeroed();
    // SAFETY: `&st` points to an initialized SYSTEMTIME and `&mut ft` is a
    // stack-local for the API to fill in. `FILETIME` has same layout as
    // type in `windows-sys`.
    unsafe {
        SystemTimeToFileTime(
            &raw const st,
            &raw mut ft as *mut windows_sys::Win32::Foundation::FILETIME,
        )
    };
    ft
}

/// Promote an existing hidden Mullvad record to the visible area.
fn promote_record(record: &mut IconStreamsRecord) {
    if record.visibility == SHOW_ICON_AND_NOTIFICATIONS {
        return;
    }

    // We don't have access to the full records slice here to compute ordinal,
    // but the caller handles ordinal assignment before calling this.
    record.visibility = SHOW_ICON_AND_NOTIFICATIONS;
    record.time1 = current_filetime();
}

/// Promote Mullvad tray icon to visible area (if needed).
pub fn promote_tray_icon() -> io::Result<()> {
    let mut streams = IconStreams::read()?;
    let new_ordinal = streams.next_free_ordinal().unwrap_or(0);

    if let Some(record) = streams.find_record(PRODUCT_NAME) {
        if record.visibility != SHOW_ICON_AND_NOTIFICATIONS {
            // Hidden record found - promote it
            promote_record(record);
            record.ordinal = new_ordinal;
            restart_explorer(streams)?;
        }
    } else {
        // No record found - inject from template
        streams
            .inject_mullvad_record()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        restart_explorer(streams)?;
    }

    Ok(())
}

/// Kill all explorer.exe instances and restart with a duplicated security token.
fn restart_explorer(streams: IconStreams) -> io::Result<()> {
    let explorer =
        get_known_folder_path(&FOLDERID_Windows, KF_FLAG_DEFAULT, None)?.join("explorer.exe");
    let explorer_pids: Vec<u32> = enum_process_ids()
        .into_iter()
        .filter(|&pid| {
            let Ok(handle) = open_process(PROCESS_QUERY_INFORMATION, pid) else {
                return false;
            };
            let Ok(path) = query_full_process_image_name(&handle) else {
                return false;
            };
            path.to_string_lossy()
                .to_lowercase()
                .contains("explorer.exe")
        })
        .collect();

    let Some(&first_pid) = explorer_pids.first() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find explorer.exe",
        ));
    };

    // Duplicate the security context from the first explorer process before killing any
    let context_token: OwnedHandle = {
        let process = open_process(PROCESS_QUERY_INFORMATION | TOKEN_QUERY, first_pid)?;
        let token =
            open_process_token(&process, TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_IMPERSONATE)?;
        duplicate_token(
            &token,
            TOKEN_ALL_ACCESS,
            SecurityImpersonation,
            TokenPrimary,
        )?
    };

    // Terminate all explorer processes
    let mut terminated = 0usize;
    for pid in explorer_pids {
        let Ok(handle) = open_process(PROCESS_TERMINATE, pid) else {
            continue;
        };
        // winlogon restarts explorer if exit code is 0, so use 1
        // SAFETY: `handle` is a live process handle.
        if unsafe { TerminateProcess(handle.as_raw_handle(), 1) } != 0 {
            // SAFETY: `handle` remains valid for this scope.
            unsafe { WaitForSingleObject(handle.as_raw_handle(), INFINITE) };
            terminated += 1;
        }
    }

    if terminated == 0 {
        return Err(io::Error::other(
            "Could not terminate any explorer.exe instance",
        ));
    }

    // Write the updated icon streams blob
    streams.write()?;

    // Restart explorer using the duplicated security context. The returned
    // process/thread handles are dropped (closed) immediately.
    let _ = create_process_with_token(&context_token, &explorer)?;

    Ok(())
}

/// Decode a ROT13-encoded null-terminated wide string.
fn decode_application_path(encoded: &[u16]) -> String {
    let decoded: Vec<u16> = encoded
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| rot13_char(c))
        .collect();
    // TODO: lossy :(
    OsString::from_wide(&decoded).to_string_lossy().into_owned()
}

/// Apply ROT13 to a single Unicode character (letters only).
///
/// Modulo applied to stay with `a..=z` or `A..=Z` range.
fn rot13_char(c: u16) -> u16 {
    const CA: u16 = b'A' as u16;
    const CZ: u16 = b'Z' as u16;
    const LA: u16 = b'a' as u16;
    const LZ: u16 = b'z' as u16;

    match c {
        CA..=CZ => {
            let shifted = c + 13;
            if shifted <= CZ {
                shifted
            } else {
                CA + (shifted - CZ - 1)
            }
        }
        LA..=LZ => {
            let shifted = c + 13;
            if shifted <= LZ {
                shifted
            } else {
                LA + (shifted - LZ - 1)
            }
        }
        _ => c,
    }
}

/// Open a process handle for `pid`, wrapped in an `OwnedHandle`.
fn open_process(access: u32, pid: u32) -> io::Result<OwnedHandle> {
    // SAFETY: `pid` is a u32; OpenProcess accepts any PID and returns null
    // if the process is gone or access is denied (checked below).
    let raw = unsafe { OpenProcess(access, FALSE, pid) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw` is a valid process handle just returned by `OpenProcess`.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}

/// Open the access token of the process referred to by `process`.
fn open_process_token(process: impl AsHandle, access: u32) -> io::Result<OwnedHandle> {
    let mut raw_token: HANDLE = ptr::null_mut();
    // SAFETY: `process` is a live process handle; `&mut raw_token` is a
    // stack-local the API fills in on success.
    let status = unsafe {
        OpenProcessToken(
            process.as_handle().as_raw_handle(),
            access,
            &raw mut raw_token,
        )
    };
    if status == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw_token` is a valid token handle filled in above.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw_token) })
}

/// Duplicate `token` with the given access, impersonation level, and type.
fn duplicate_token(
    token: impl AsHandle,
    access: u32,
    impersonation_level: SECURITY_IMPERSONATION_LEVEL,
    token_type: TOKEN_TYPE,
) -> io::Result<OwnedHandle> {
    let mut raw_dup: HANDLE = ptr::null_mut();
    // SAFETY: `token` is a live token handle; null SECURITY_ATTRIBUTES is
    // allowed; `&mut raw_dup` is a stack-local the API fills in on success.
    let status = unsafe {
        DuplicateTokenEx(
            token.as_handle().as_raw_handle(),
            access,
            ptr::null(),
            impersonation_level,
            token_type,
            &raw mut raw_dup,
        )
    };
    if status == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw_dup` is a valid duplicated token filled in above.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw_dup) })
}

/// Owned handles to a created process and its primary thread.
#[expect(dead_code)]
struct ProcessInfo {
    process: OwnedHandle,
    thread: OwnedHandle,
}

/// Spawn `application` under the given user `token`.
fn create_process_with_token(token: impl AsHandle, application: &Path) -> io::Result<ProcessInfo> {
    if !application.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path needs to be absolute",
        ));
    }

    // CreateProcessWithTokenW needs a quoted command line for argv[0].
    let mut quoted = OsString::from("\"");
    quoted.push(application);
    quoted.push("\"");
    let mut cmd_wide = U16CString::from_os_str_truncate(&quoted).into_vec_with_nul();

    let startup_info = STARTUPINFOW {
        cb: mem::size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();

    // SAFETY: `token` is valid; `cmd_wide` is null-terminated, and
    // the other arguments are valid pointers or null.
    let status = unsafe {
        CreateProcessWithTokenW(
            token.as_handle().as_raw_handle(),
            0,
            ptr::null(),
            cmd_wide.as_mut_ptr(),
            0,
            ptr::null(),
            ptr::null(),
            &raw const startup_info,
            &raw mut process_info,
        )
    };

    if status == 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: handle was just produced by `CreateProcessWithTokenW`. Caller is responsible for it.
    let process = unsafe { OwnedHandle::from_raw_handle(process_info.hProcess) };
    // SAFETY: handle was just produced by `CreateProcessWithTokenW`. Caller is responsible for it.
    let thread = unsafe { OwnedHandle::from_raw_handle(process_info.hThread) };
    Ok(ProcessInfo { process, thread })
}

/// Get the full image path of the process referred to by `handle`.
fn query_full_process_image_name(handle: impl AsHandle) -> io::Result<OsString> {
    let mut path = [0u16; MAX_PATH as usize];
    let mut size = path.len() as u32;
    // SAFETY: `handle` is a live process handle; `&mut size` carries the
    // capacity of `path` and is updated to the length written.
    if unsafe {
        QueryFullProcessImageNameW(
            handle.as_handle().as_raw_handle(),
            0,
            path.as_mut_ptr(),
            &raw mut size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }

    Ok(OsString::from_wide(&path[..size as usize]))
}

/// Enumerate all process IDs currently running on the system.
fn enum_process_ids() -> Vec<u32> {
    let mut pid_buf = vec![0u32; 2048];
    let mut bytes_written: u32 = 0;
    let bytes_available = (pid_buf.len() * mem::size_of::<u32>()) as u32;

    // SAFETY: `pid_buf` is `bytes_available` writable bytes and
    // `&mut bytes_written` is a stack-local.
    if unsafe {
        EnumProcesses(
            pid_buf.as_mut_ptr(),
            bytes_available,
            &raw mut bytes_written,
        )
    } == 0
    {
        return vec![];
    }

    let num_procs = bytes_written as usize / mem::size_of::<u32>();
    pid_buf.truncate(num_procs);
    pid_buf
}
