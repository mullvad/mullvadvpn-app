//! Tray icon management for the Mullvad VPN installer.
//!
//! Exports:
//! - `PromoteTrayIcon` - ensure the Mullvad VPN tray icon is visible in the notification area

use std::ffi::OsString;
use std::io;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

use nsis_plugin_api::{nsis_fn, pushint, pushstr};

use crate::NsisStatus;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, FALSE, FILETIME, HANDLE, MAX_PATH};
use windows_sys::Win32::Security::{
    DuplicateTokenEx, SecurityImpersonation, TOKEN_ALL_ACCESS, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
    TOKEN_QUERY, TokenPrimary,
};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_BINARY, RegCloseKey, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessAsUserW, INFINITE, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, STARTUPINFOW, TerminateProcess,
    WaitForSingleObject,
};
use windows_sys::Win32::UI::Shell::{FOLDERID_Windows, KF_FLAG_DEFAULT, SHGetKnownFolderPath};

// Template for a new Mullvad VPN tray record (embedded at compile time)
static MULLVAD_TRAY_RECORD_TEMPLATE: &[u8] = include_bytes!("mullvad_tray_record.bin");

/// ICON_STREAMS_HEADER (packed, 20 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IconStreamsHeader {
    header_size: u32,
    u1: u32,
    u2: u16,
    u3: u16,
    number_records: u32,
    offset_first_record: u32,
}

/// ICON_STREAMS_RECORD union details variant
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct DetailsVariant {
    application_name: [u16; 257],
    padding: [u8; 6],
}

/// ICON_STREAMS_RECORD union extended_details variant
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ExtendedDetailsVariant {
    u12: u32,
    u13: u16,
    application_name: [u16; 257],
}

const _: () = assert!(mem::size_of::<DetailsVariant>() == mem::size_of::<ExtendedDetailsVariant>());

/// ICON_STREAMS_RECORD union
#[repr(C, packed)]
#[derive(Clone, Copy)]
union DummyUnion {
    details: DetailsVariant,
    extended_details: ExtendedDetailsVariant,
}

/// Tray icon visibility constants
const SHOW_ICON_AND_NOTIFICATIONS: u32 = 2;

/// ICON_STREAMS_RECORD (packed, 1640 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IconStreamsRecord {
    application_path: [u16; MAX_PATH as usize],
    u1: u32,
    u2: u32,
    visibility: u32,
    year_created: u16,
    month_created: u16,
    last_tooltip: [u16; MAX_PATH as usize],
    u6: u32,
    u7: u32,
    imagelist_id: u32,
    guid: [u8; 16],
    u8_: u32,
    u9: u32,
    u10: u32,
    time1: FILETIME,
    time2: FILETIME,
    u11: u32,
    dummy_union: DummyUnion,
    ordinal: u32,
}

const HEADER_SIZE: usize = mem::size_of::<IconStreamsHeader>();
const RECORD_SIZE: usize = mem::size_of::<IconStreamsRecord>();

// Compile-time size verification
const _: () = assert!(HEADER_SIZE == 20);
const _: () = assert!(RECORD_SIZE == 1640);

/// Apply ROT13 to a single Unicode character (letters only).
fn rot13_char(c: u16) -> u16 {
    let ca = b'A' as u16;
    let cz = b'Z' as u16;
    let la = b'a' as u16;
    let lz = b'z' as u16;

    if c >= ca && c <= cz {
        let shifted = c + 13;
        if shifted <= cz {
            shifted
        } else {
            ca + (shifted - cz - 1)
        }
    } else if c >= la && c <= lz {
        let shifted = c + 13;
        if shifted <= lz {
            shifted
        } else {
            la + (shifted - lz - 1)
        }
    } else {
        c
    }
}

/// Decode a ROT13-encoded null-terminated wide string.
fn decode_application_path(encoded: &[u16]) -> String {
    let decoded: Vec<u16> = encoded
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| rot13_char(c))
        .collect();
    OsString::from_wide(&decoded).to_string_lossy().into_owned()
}

/// Parse the binary IconStreams registry blob into header + records.
fn parse_blob(blob: &[u8]) -> io::Result<(IconStreamsHeader, Vec<IconStreamsRecord>)> {
    if blob.len() < HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "IconStreams blob too small for header",
        ));
    }

    // SAFETY: we verified `blob.len() >= HEADER_SIZE` (= sizeof Header), so
    // the read of `IconStreamsHeader` is in-bounds; `read_unaligned` handles
    // any alignment.
    let header: IconStreamsHeader = unsafe { ptr::read_unaligned(blob.as_ptr() as *const _) };
    let header_size = { header.header_size } as usize;
    let num_records = { header.number_records } as usize;
    let offset = { header.offset_first_record } as usize;

    if header_size != HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "IconStreams header size mismatch",
        ));
    }

    if num_records == 0 {
        return Ok((header, vec![]));
    }

    if blob.len() < HEADER_SIZE + RECORD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "IconStreams blob too small for records",
        ));
    }

    let expected_size = offset + num_records * RECORD_SIZE;
    if blob.len() != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "IconStreams blob size mismatch: expected {expected_size}, got {}",
                blob.len()
            ),
        ));
    }

    let records: Vec<IconStreamsRecord> = (0..num_records)
        .map(|i| {
            let record_ptr = blob[offset + i * RECORD_SIZE..].as_ptr() as *const IconStreamsRecord;
            // SAFETY: the size check above ensures
            // `offset + i * RECORD_SIZE + RECORD_SIZE <= blob.len()`, so the
            // read is in-bounds; `read_unaligned` handles any alignment.
            unsafe { ptr::read_unaligned(record_ptr) }
        })
        .collect();

    Ok((header, records))
}

/// Serialize header + records back into a binary blob.
fn pack_blob(header: &IconStreamsHeader, records: &[IconStreamsRecord]) -> Vec<u8> {
    let mut packed_header = *header;
    packed_header.header_size = HEADER_SIZE as u32;
    packed_header.number_records = records.len() as u32;
    packed_header.offset_first_record = HEADER_SIZE as u32;

    let total = HEADER_SIZE + records.len() * RECORD_SIZE;
    let mut blob = vec![0u8; total];

    // SAFETY: `blob` is `total >= HEADER_SIZE` bytes, so the write is in-
    // bounds; `write_unaligned` handles alignment.
    unsafe {
        ptr::write_unaligned(blob.as_mut_ptr() as *mut IconStreamsHeader, packed_header);
    }

    for (i, record) in records.iter().enumerate() {
        let offset = HEADER_SIZE + i * RECORD_SIZE;
        // SAFETY: `total = HEADER_SIZE + records.len() * RECORD_SIZE`, so
        // each record write at `offset..offset+RECORD_SIZE` is in-bounds.
        unsafe {
            ptr::write_unaligned(
                blob[offset..].as_mut_ptr() as *mut IconStreamsRecord,
                *record,
            );
        }
    }

    blob
}

/// Find the record whose decoded ApplicationPath contains `substring`.
fn find_record<'a>(
    records: &'a mut [IconStreamsRecord],
    substring: &str,
) -> Option<&'a mut IconStreamsRecord> {
    records.iter_mut().find(|r| {
        let path = decode_application_path(&{ r.application_path });
        path.contains(substring)
    })
}

/// Get the highest ordinal among records in the given visibility group.
/// Returns 0 if no matching records exist.
fn next_free_ordinal(records: &[IconStreamsRecord], visible: bool) -> u32 {
    let mut highest: u32 = 0;
    let mut found = false;

    for record in records {
        let vis = { record.visibility };
        let matches = if visible {
            vis == SHOW_ICON_AND_NOTIFICATIONS
        } else {
            vis != SHOW_ICON_AND_NOTIFICATIONS
        };

        if matches {
            found = true;
            let ord = { record.ordinal };
            if ord > highest {
                highest = ord;
            }
        }
    }

    if found { highest + 1 } else { 0 }
}

/// Get the current system time as a FILETIME.
fn current_filetime() -> FILETIME {
    use windows_sys::Win32::System::SystemInformation::GetSystemTime;
    use windows_sys::Win32::System::Time::SystemTimeToFileTime;

    let mut st = windows_sys::Win32::Foundation::SYSTEMTIME::default();

    // SAFETY: `&mut st` points to a stack-local SYSTEMTIME the API fills in.
    unsafe { GetSystemTime(&raw mut st) };

    let mut ft = FILETIME::default();
    // SAFETY: `&st` points to an initialized SYSTEMTIME and `&mut ft` is a
    // stack-local for the API to fill in.
    unsafe { SystemTimeToFileTime(&raw const st, &raw mut ft) };
    ft
}

/// Inject a new Mullvad tray record using the embedded template.
fn inject_mullvad_record(records: &mut Vec<IconStreamsRecord>) -> io::Result<()> {
    if MULLVAD_TRAY_RECORD_TEMPLATE.len() != RECORD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Mullvad tray record template has wrong size",
        ));
    }

    // SAFETY: we verified the template has exactly `RECORD_SIZE` bytes, so
    // the read is in-bounds; `read_unaligned` handles alignment.
    let mut new_record: IconStreamsRecord =
        unsafe { ptr::read_unaligned(MULLVAD_TRAY_RECORD_TEMPLATE.as_ptr() as *const _) };

    let now = current_filetime();
    let ordinal = next_free_ordinal(records, true);

    // Get current system time for year/month fields
    let mut st = windows_sys::Win32::Foundation::SYSTEMTIME::default();
    // SAFETY: `&mut st` points to a stack-local SYSTEMTIME the API fills in.
    unsafe { windows_sys::Win32::System::SystemInformation::GetSystemTime(&raw mut st) };

    new_record.visibility = SHOW_ICON_AND_NOTIFICATIONS;
    new_record.year_created = st.wYear;
    new_record.month_created = st.wMonth;
    new_record.u7 = 0;
    new_record.imagelist_id = 0xFFFFFFFF;
    new_record.time1 = now;
    new_record.time2 = FILETIME::default();
    new_record.ordinal = ordinal;

    records.push(new_record);
    Ok(())
}

/// Promote an existing hidden Mullvad record to the visible area.
fn promote_record(record: &mut IconStreamsRecord) {
    if { record.visibility } == SHOW_ICON_AND_NOTIFICATIONS {
        return;
    }

    // We don't have access to the full records slice here to compute ordinal,
    // but the caller handles ordinal assignment before calling this.
    record.visibility = SHOW_ICON_AND_NOTIFICATIONS;
    record.time1 = current_filetime();
}

// Registry key / value for tray icon data
const TRAY_KEY_NAME: &[u8] =
    b"Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\TrayNotify\0";
const TRAY_VALUE_NAME: &[u8] = b"IconStreams\0";

fn to_wide_nul(s: &[u8]) -> Vec<u16> {
    s.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u16)
        .chain(std::iter::once(0))
        .collect()
}

/// Open the TrayNotify registry key.
fn open_tray_key(write: bool) -> io::Result<RegKeyGuard> {
    let key_name = to_wide_nul(TRAY_KEY_NAME);
    let access = if write {
        KEY_READ | KEY_WRITE
    } else {
        KEY_READ
    };

    let mut hkey: HKEY = ptr::null_mut();
    // SAFETY: `key_name` is a null-terminated wide string built above; the
    // out-pointer is a stack-local.
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            key_name.as_ptr(),
            0,
            access,
            &raw mut hkey,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(RegKeyGuard(hkey))
}

struct RegKeyGuard(HKEY);

impl Drop for RegKeyGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid HKEY produced by `RegOpenKeyExW`,
        // owned uniquely by this guard.
        unsafe { RegCloseKey(self.0) };
    }
}

/// Read the IconStreams binary blob from the registry.
fn read_icon_streams(key: &RegKeyGuard) -> io::Result<Vec<u8>> {
    let val_name = to_wide_nul(TRAY_VALUE_NAME);
    let mut value_type: u32 = 0;
    let mut buf_size: u32 = 0;

    // SAFETY: `key.0` is a live HKEY, `val_name` is a null-terminated wide
    // string, and the buffer pointer is null so only `buf_size` is written.
    unsafe {
        RegQueryValueExW(
            key.0,
            val_name.as_ptr(),
            ptr::null(),
            &raw mut value_type,
            ptr::null_mut(),
            &raw mut buf_size,
        )
    };

    if buf_size == 0 {
        return Ok(vec![]);
    }

    let mut buf = vec![0u8; buf_size as usize];
    // SAFETY: `key.0` is a live HKEY, `val_name` is a null-terminated wide
    // string, `buf` is `buf_size` writable bytes, and `&mut buf_size` carries
    // the buffer capacity to the API.
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            val_name.as_ptr(),
            ptr::null(),
            &raw mut value_type,
            buf.as_mut_ptr(),
            &raw mut buf_size,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }

    buf.truncate(buf_size as usize);
    Ok(buf)
}

/// Write the IconStreams binary blob to the registry.
fn write_icon_streams(key: &RegKeyGuard, blob: &[u8]) -> io::Result<()> {
    let val_name = to_wide_nul(TRAY_VALUE_NAME);

    // SAFETY: `key.0` is a live HKEY, `val_name` is a null-terminated wide
    // string, and `blob` is `blob.len()` valid bytes.
    let result = unsafe {
        RegSetValueExW(
            key.0,
            val_name.as_ptr(),
            0,
            REG_BINARY,
            blob.as_ptr(),
            blob.len() as u32,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

/// Find all process IDs with the given executable name (case-insensitive substring match on path).
fn find_process_ids_by_name(name: &str) -> Vec<u32> {
    use windows_sys::Win32::System::ProcessStatus::EnumProcesses;
    use windows_sys::Win32::System::Threading::QueryFullProcessImageNameW;

    let mut pid_buf = vec![0u32; 2048];
    let mut bytes_written: u32 = 0;
    let bytes_available = (pid_buf.len() * mem::size_of::<u32>()) as u32;

    // SAFETY: `pid_buf` is `bytes_available` writable bytes (2048 u32s) and
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

    let name_lower = name.to_lowercase();

    pid_buf
        .into_iter()
        .filter(|&pid| {
            // SAFETY: `pid` is a u32 from `EnumProcesses`; the API tolerates
            // dead PIDs by returning a null handle (handled below).
            let raw = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid) };
            if raw.is_null() {
                return false;
            }
            // SAFETY: `raw` is a valid process handle just returned by
            // `OpenProcess`, transferred to `_handle`.
            let _handle = unsafe { OwnedHandle::from_raw_handle(raw.cast()) };

            let mut path = [0u16; 1024];
            let mut size = path.len() as u32;
            // SAFETY: `raw` is the live process handle owned by `_handle`;
            // `path` is 1024 writable u16s; `&mut size` carries capacity.
            if unsafe { QueryFullProcessImageNameW(raw, 0, path.as_mut_ptr(), &raw mut size) } == 0
            {
                return false;
            }

            let path_str = OsString::from_wide(&path[..size as usize])
                .to_string_lossy()
                .to_lowercase();
            path_str.contains(&name_lower)
        })
        .collect()
}

/// Get the path to explorer.exe from the Windows directory.
fn explorer_path() -> io::Result<Vec<u16>> {
    let mut path_ptr: windows_sys::core::PWSTR = ptr::null_mut();
    // SAFETY: `&FOLDERID_Windows` is a valid KNOWNFOLDERID static; null
    // token uses the calling thread's identity; `&mut path_ptr` is filled in
    // by the API with a CoTaskMem-allocated PWSTR.
    let status = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_Windows,
            KF_FLAG_DEFAULT as u32,
            ptr::null_mut(),
            &raw mut path_ptr,
        )
    };

    if status != 0 {
        return Err(io::Error::from_raw_os_error(status));
    }

    let len = {
        let mut i = 0;
        // SAFETY: `path_ptr` is a non-null null-terminated wide string
        // returned by `SHGetKnownFolderPath`; loop stops at the terminator.
        while unsafe { *path_ptr.add(i) } != 0 {
            i += 1;
        }
        i
    };
    // SAFETY: `path_ptr` points to at least `len` u16s (verified above) and
    // remains valid until `CoTaskMemFree` below.
    let wide = unsafe { std::slice::from_raw_parts(path_ptr, len) };
    let mut path: Vec<u16> = wide.to_vec();
    // SAFETY: `path_ptr` was allocated by `SHGetKnownFolderPath` (CoTaskMem)
    // and is freed exactly once here.
    unsafe { CoTaskMemFree(path_ptr as *mut _) };

    // Append \explorer.exe\0
    path.extend_from_slice(&[
        b'\\' as u16,
        b'e' as u16,
        b'x' as u16,
        b'p' as u16,
        b'l' as u16,
        b'o' as u16,
        b'r' as u16,
        b'e' as u16,
        b'r' as u16,
        b'.' as u16,
        b'e' as u16,
        b'x' as u16,
        b'e' as u16,
        0,
    ]);
    Ok(path)
}

/// Kill all explorer.exe instances and restart with a duplicated security token.
fn restart_explorer(key: &RegKeyGuard, blob: &[u8]) -> io::Result<()> {
    let explorer = explorer_path()?;
    let pids = find_process_ids_by_name("explorer.exe");

    if pids.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find explorer.exe",
        ));
    }

    // Duplicate the security context from the first explorer process before killing any
    let context_token: OwnedHandle = {
        // SAFETY: `pids[0]` came from `EnumProcesses`; OpenProcess accepts
        // any PID and returns null if the process is gone (handled below).
        let raw_proc =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | TOKEN_QUERY, FALSE, pids[0]) };
        if raw_proc.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `raw_proc` is a valid process handle just returned.
        let process = unsafe { OwnedHandle::from_raw_handle(raw_proc.cast()) };

        let mut raw_token: HANDLE = ptr::null_mut();
        // SAFETY: `process` is a live process handle; `&mut raw_token` is a
        // stack-local for the API to fill in.
        if unsafe {
            OpenProcessToken(
                process.as_raw_handle().cast(),
                TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_IMPERSONATE,
                &raw mut raw_token,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `raw_token` is a valid token handle filled in above.
        let token = unsafe { OwnedHandle::from_raw_handle(raw_token.cast()) };

        let mut raw_dup: HANDLE = ptr::null_mut();
        // SAFETY: `token` is a valid token handle; null SECURITY_ATTRIBUTES
        // is allowed; `&mut raw_dup` is filled in by the API.
        if unsafe {
            DuplicateTokenEx(
                token.as_raw_handle().cast(),
                TOKEN_ALL_ACCESS,
                ptr::null(),
                SecurityImpersonation,
                TokenPrimary,
                &raw mut raw_dup,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `raw_dup` is a valid duplicated token filled in above.
        unsafe { OwnedHandle::from_raw_handle(raw_dup.cast()) }
    };

    // Terminate all explorer processes
    let mut terminated = 0usize;
    for pid in &pids {
        // SAFETY: `*pid` is a u32 from `EnumProcesses`; OpenProcess returns
        // null for dead PIDs (handled below).
        let raw = unsafe { OpenProcess(PROCESS_TERMINATE, FALSE, *pid) };
        if raw.is_null() {
            continue;
        }
        // SAFETY: `raw` is a valid process handle just returned.
        let _handle = unsafe { OwnedHandle::from_raw_handle(raw.cast()) };
        // winlogon restarts explorer if exit code is 0, so use 1
        // SAFETY: `raw` is the live process handle owned by `_handle`.
        if unsafe { TerminateProcess(raw, 1) } != 0 {
            // SAFETY: `raw` remains valid until `_handle` is dropped.
            unsafe { WaitForSingleObject(raw, INFINITE) };
            terminated += 1;
        }
    }

    if terminated == 0 {
        return Err(io::Error::other(
            "Could not terminate any explorer.exe instance",
        ));
    }

    // Write the updated icon streams blob
    write_icon_streams(key, blob)?;

    // Restart explorer using the duplicated security context
    let startup_info = STARTUPINFOW {
        cb: mem::size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };

    let mut process_info = PROCESS_INFORMATION::default();

    let mut explorer_cmd = explorer.clone();
    // CreateProcessAsUserW needs a mutable command line
    // SAFETY: `context_token` is a valid duplicated token; `explorer` and
    // `explorer_cmd` are null-terminated wide strings; the security/
    // environment/dir pointers are null (allowed); `&startup_info` and
    // `&mut process_info` are stack-locals.
    let success = unsafe {
        CreateProcessAsUserW(
            context_token.as_raw_handle().cast(),
            explorer.as_ptr(),
            explorer_cmd.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            FALSE,
            0,
            ptr::null(),
            ptr::null(),
            &raw const startup_info,
            &raw mut process_info,
        )
    };

    if success != 0 {
        // SAFETY: both handles were just produced by `CreateProcessAsUserW`;
        // wrapping them in `OwnedHandle` closes them on drop.
        unsafe {
            let _ = OwnedHandle::from_raw_handle(process_info.hProcess.cast());
            let _ = OwnedHandle::from_raw_handle(process_info.hThread.cast());
        }
    }

    Ok(())
}

/// Main implementation of PromoteTrayIcon.
fn promote_tray_icon() -> io::Result<()> {
    let key = open_tray_key(true)?;
    let blob = read_icon_streams(&key)?;

    if blob.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "IconStreams registry value is empty",
        ));
    }

    let (header, mut records) = parse_blob(&blob)?;

    let mut update_registry = true;

    if let Some(record) = find_record(&mut records, "Mullvad VPN") {
        if { record.visibility } == SHOW_ICON_AND_NOTIFICATIONS {
            // Already visible - no update needed
            update_registry = false;
        } else {
            // Hidden record found - promote it
            let new_ordinal = next_free_ordinal(&records, true);
            let record = find_record(&mut records, "Mullvad VPN").unwrap();
            promote_record(record);
            record.ordinal = new_ordinal;
        }
    } else {
        // No record found - inject from template
        inject_mullvad_record(&mut records)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    }

    if update_registry {
        let new_blob = pack_blob(&header, &records);
        restart_explorer(&key, &new_blob)?;
    }

    Ok(())
}

// PromoteTrayIcon
//
// Ensure the Mullvad VPN tray icon is in the visible notification area.
// Pushes error message and status code.
#[nsis_fn]
fn PromoteTrayIcon() -> Result<(), nsis_plugin_api::Error> {
    let (message, status) = match promote_tray_icon() {
        Ok(()) => (String::new(), NsisStatus::Success),
        Err(e) => (e.to_string(), NsisStatus::GeneralError),
    };
    // SAFETY: `exdll_init` was called.
    unsafe {
        pushstr(&message)?;
        pushint(status as i32)
    }
}
