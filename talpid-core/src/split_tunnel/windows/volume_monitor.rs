//! Used to monitor volume mounts and dismounts, and reapply the split
//! tunnel config if any of the excluded paths are affected by them.
use super::path_monitor::PathMonitorHandle;
use crate::window::{create_hidden_window, WindowCloseHandle};
use futures::{channel::mpsc, StreamExt};
use std::{
    ffi::OsString,
    io,
    path::{self, Path},
    sync::{mpsc as sync_mpsc, Arc, Mutex, MutexGuard},
};
use talpid_types::ErrorExt;
use windows_sys::Win32::{
    Storage::FileSystem::GetLogicalDrives,
    System::SystemServices::{
        DBTF_NET, DBT_DEVICEARRIVAL, DBT_DEVICEREMOVECOMPLETE, DBT_DEVTYP_VOLUME,
        DEV_BROADCAST_HDR, DEV_BROADCAST_VOLUME,
    },
    UI::WindowsAndMessaging::{DefWindowProcW, WM_DEVICECHANGE},
};

pub(super) struct VolumeMonitor(());

pub(super) struct VolumeMonitorHandle {
    window_handle: WindowCloseHandle,
    internal_monitor_task: tokio::task::JoinHandle<()>,
}

impl Drop for VolumeMonitorHandle {
    fn drop(&mut self) {
        self.window_handle.close();
        self.internal_monitor_task.abort();
    }
}

impl VolumeMonitor {
    pub fn spawn(
        path_monitor: PathMonitorHandle,
        update_tx: sync_mpsc::Sender<()>,
        paths: Arc<Mutex<Vec<OsString>>>,
        volume_update_rx: mpsc::UnboundedReceiver<()>,
    ) -> VolumeMonitorHandle {
        // A bitmask containing all (known) mounted drives.
        let known_state = Arc::new(Mutex::new(0u32));

        // Lock before registering event handler
        let mut known_state_guard = known_state.lock().unwrap();

        let internal_monitor_task = tokio::spawn(frontend_monitor(
            known_state.clone(),
            path_monitor.clone(),
            update_tx.clone(),
            paths.clone(),
            volume_update_rx,
        ));

        let window_handle =
            start_internal_monitor(known_state.clone(), path_monitor, update_tx, paths);

        *known_state_guard = get_logical_drives().unwrap_or_else(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to initialize state of mounted volumes")
            );
            0
        });

        VolumeMonitorHandle {
            window_handle,
            internal_monitor_task,
        }
    }
}

/// Monitors update requests from frontends. This checks if the known state of mounted volumes
/// has change, and, if so, reapplies the ST config.
async fn frontend_monitor(
    known_state: Arc<Mutex<u32>>,
    path_monitor: PathMonitorHandle,
    update_tx: sync_mpsc::Sender<()>,
    paths: Arc<Mutex<Vec<OsString>>>,
    mut volume_update_rx: mpsc::UnboundedReceiver<()>,
) {
    while let Some(()) = volume_update_rx.next().await {
        let mut known_state_guard = known_state.lock().unwrap();
        let new_state = get_logical_drives().unwrap_or_else(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to obtain new state of mounted volumes")
            );
            *known_state_guard
        });

        // Was there a change?
        let state_diff = *known_state_guard ^ new_state;
        if state_diff != 0 {
            *known_state_guard = new_state;
            let paths_guard = paths.lock().unwrap();
            if matches_volume(state_diff, &paths_guard) {
                // Reapply config
                let _ = update_tx.send(());
                let _ = path_monitor.refresh();
            }
        }
    }
}

/// Monitors window events received by session 0.
fn start_internal_monitor(
    known_state: Arc<Mutex<u32>>,
    path_monitor: PathMonitorHandle,
    update_tx: sync_mpsc::Sender<()>,
    paths: Arc<Mutex<Vec<OsString>>>,
) -> WindowCloseHandle {
    create_hidden_window(move |window, message, w_param, l_param| {
        if !is_device_arrival_or_removal(message, w_param) {
            return unsafe { DefWindowProcW(window, message, w_param, l_param) };
        }
        let paths_guard = paths.lock().unwrap();
        let mut known_state_guard = known_state.lock().unwrap();

        let volumes = unsafe { parse_device_volume_broadcast(&*(l_param as *const _)) };

        let prev_state = *known_state_guard;
        let is_arrival = w_param == DBT_DEVICEARRIVAL as usize;
        if is_arrival {
            *known_state_guard |= volumes;
        } else {
            *known_state_guard &= !volumes;
        }

        // Compare against known state to ignore duplicate notifications
        // from frontends
        let state_diff = *known_state_guard ^ prev_state;
        if state_diff != 0 {
            if matches_volume(volumes, &paths_guard) {
                // Reapply config
                let _ = update_tx.send(());
                let _ = path_monitor.refresh();
            }
        }

        // Always grant the request
        1
    })
}

/// Return a bitmask representing all currently available disk drives.
/// Each bit refers to a volume letter. The bit 0 refers to 'A', bit 1
/// refers to 'B', bit 2 to 'C', etc.
fn get_logical_drives() -> io::Result<u32> {
    let result = unsafe { GetLogicalDrives() };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result)
}

/// Return whether any of the paths in `paths_guard` reside on any volume in `volumes` (a mask).
fn matches_volume(volumes: u32, paths_guard: &MutexGuard<'_, Vec<OsString>>) -> bool {
    for path in &**paths_guard {
        let path = (path as &dyn AsRef<Path>).as_ref();
        if let Some(path::Component::Prefix(prefix)) = path.components().next() {
            match prefix.kind() {
                path::Prefix::VerbatimDisk(disk) | path::Prefix::Disk(disk) => {
                    if disk < 'A' as u8 || disk > 'Z' as u8 {
                        log::warn!("Ignoring invalid volume \"{}\"", disk as char);
                        continue;
                    }
                    let disk = disk - 'A' as u8;
                    if volumes & (1 << disk) != 0 {
                        return true;
                    }
                }
                _ => (),
            }
        }
    }
    false
}

fn is_device_arrival_or_removal(message: u32, w_param: usize) -> bool {
    message == WM_DEVICECHANGE
        && (w_param == DBT_DEVICEARRIVAL as usize || w_param == DBT_DEVICEREMOVECOMPLETE as usize)
}

/// Return volumes affected by the device arrival or removal message as a mask.
/// This has the same format as `get_logical_drives()`.
unsafe fn parse_device_volume_broadcast(broadcast: &DEV_BROADCAST_HDR) -> u32 {
    if broadcast.dbch_devicetype != DBT_DEVTYP_VOLUME {
        return 0;
    }

    let volume_broadcast = &*(broadcast as *const _ as *const DEV_BROADCAST_VOLUME);
    if volume_broadcast.dbcv_flags & DBTF_NET != 0 {
        // Ignore net event
        return 0;
    }

    volume_broadcast.dbcv_unitmask
}
