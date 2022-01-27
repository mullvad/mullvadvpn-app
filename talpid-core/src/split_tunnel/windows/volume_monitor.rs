//! Used to monitor volume mounts and dismounts, and reapply the split
//! tunnel config if any of the excluded paths are affected by them.
use crate::windows::window::{create_hidden_window, WindowCloseHandle};
use std::{
    ffi::OsString,
    path::{self, Path},
    sync::{mpsc as sync_mpsc, Arc, Mutex},
};
use winapi::{
    shared::minwindef::TRUE,
    um::{
        dbt::{
            DBTF_NET, DBT_DEVICEARRIVAL, DBT_DEVICEREMOVECOMPLETE, DBT_DEVTYP_VOLUME,
            DEV_BROADCAST_HDR, DEV_BROADCAST_VOLUME, WM_DEVICECHANGE,
        },
        winuser::DefWindowProcW,
    },
};

pub(super) struct VolumeMonitor(());

impl VolumeMonitor {
    pub fn spawn(
        update_tx: sync_mpsc::Sender<()>,
        paths: Arc<Mutex<Vec<OsString>>>,
    ) -> WindowCloseHandle {
        create_hidden_window(move |window, message, w_param, l_param| {
            if message != WM_DEVICECHANGE
                || (w_param != DBT_DEVICEARRIVAL && w_param != DBT_DEVICEREMOVECOMPLETE)
            {
                return unsafe { DefWindowProcW(window, message, w_param, l_param) };
            }

            let paths_guard = paths.lock().unwrap();
            let mut label_found = false;

            let volumes = unsafe { parse_broadcast(&*(l_param as *const _)) };
            for volume in volumes {
                for path in &*paths_guard {
                    let path = (path as &dyn AsRef<Path>).as_ref();
                    if let Some(path::Component::Prefix(prefix)) = path.components().next() {
                        match prefix.kind() {
                            path::Prefix::VerbatimDisk(disk) | path::Prefix::Disk(disk) => {
                                if disk == volume {
                                    label_found = true;
                                    break;
                                }
                            }
                            _ => (),
                        }
                    }
                }
                if label_found {
                    break;
                }
            }

            if label_found {
                // Reapply config
                let _ = update_tx.send(());
            }

            // Always grant the request
            TRUE as isize
        })
    }
}

/// Return volume labels (ASCII-encoded) affected by the device arrival or removal message, if any.
unsafe fn parse_broadcast(broadcast: &DEV_BROADCAST_HDR) -> Vec<u8> {
    let mut labels = vec![];

    if broadcast.dbch_devicetype != DBT_DEVTYP_VOLUME {
        return labels;
    }

    let volume_broadcast = &*(broadcast as *const _ as *const DEV_BROADCAST_VOLUME);
    if volume_broadcast.dbcv_flags & DBTF_NET != 0 {
        // Ignore net event
        return labels;
    }

    // 26 = 1 + 'Z' - 'A'
    let num_drives = 1 + 'Z' as u8 - 'A' as u8;
    for i in 0..num_drives {
        let is_affected = ((volume_broadcast.dbcv_unitmask >> i) & 1) != 0;
        if is_affected {
            labels.push('A' as u8 + i);
        }
    }

    labels
}
