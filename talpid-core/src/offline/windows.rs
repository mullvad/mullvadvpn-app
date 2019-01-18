//! # License
//!
//! Copyright (C) 2018  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;
use log::debug;
use std::{
    ffi::c_void,
    mem::zeroed,
    os::windows::io::{IntoRawHandle, RawHandle},
    ptr, thread,
    time::Duration,
};
use winapi::{
    shared::{
        basetsd::LONG_PTR,
        minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        handleapi::CloseHandle,
        libloaderapi::GetModuleHandleW,
        processthreadsapi::GetThreadId,
        synchapi::WaitForSingleObject,
        winbase::INFINITE,
        winuser::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            GetWindowLongPtrW, PostQuitMessage, PostThreadMessageW, SetWindowLongPtrW,
            GWLP_USERDATA, GWLP_WNDPROC, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, WM_DESTROY,
            WM_POWERBROADCAST, WM_USER,
        },
    },
};

const CLASS_NAME: &[u8] = b"S\0T\0A\0T\0I\0C\0\0\0";
const REQUEST_THREAD_SHUTDOWN: UINT = WM_USER + 1;

error_chain! {
    errors {
        ThreadCreationError {
            description("Unable to create listener thread")
        }
    }
}

pub struct BroadcastListener {
    thread_handle: RawHandle,
    thread_id: DWORD,
}

unsafe impl Send for BroadcastListener {}

impl BroadcastListener {
    pub fn start<F>(client_callback: F) -> Result<Self>
    where
        F: Fn(UINT, WPARAM, LPARAM) + 'static + Send,
    {
        let join_handle = thread::Builder::new()
            .spawn(move || unsafe {
                Self::message_pump(client_callback);
            })
            .chain_err(|| ErrorKind::ThreadCreationError)?;

        let real_handle = join_handle.into_raw_handle();

        Ok(BroadcastListener {
            thread_handle: real_handle,
            thread_id: unsafe { GetThreadId(real_handle) },
        })
    }

    unsafe fn message_pump<F>(client_callback: F)
    where
        F: Fn(UINT, WPARAM, LPARAM),
    {
        let dummy_window = CreateWindowExW(
            0,
            CLASS_NAME.as_ptr() as *const u16,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            GetModuleHandleW(ptr::null_mut()),
            ptr::null_mut(),
        );

        // Move callback information to the heap.
        // This enables us to reach the callback through a "thin pointer".
        let boxed_callback = Box::new(client_callback);

        // Detach callback from Box.
        let raw_callback = Box::into_raw(boxed_callback) as *mut c_void;

        SetWindowLongPtrW(dummy_window, GWLP_USERDATA, raw_callback as LONG_PTR);
        SetWindowLongPtrW(
            dummy_window,
            GWLP_WNDPROC,
            Self::window_procedure::<F> as LONG_PTR,
        );

        let mut msg = zeroed();

        loop {
            let status = GetMessageW(&mut msg, 0 as HWND, 0, 0);

            if status < 0 {
                continue;
            }
            if status == 0 {
                break;
            }

            if msg.hwnd.is_null() {
                if msg.message == REQUEST_THREAD_SHUTDOWN {
                    DestroyWindow(dummy_window);
                }
            } else {
                DispatchMessageW(&mut msg);
            }
        }

        // Reattach callback to Box for proper clean-up.
        let _ = Box::from_raw(raw_callback as *mut F);
    }

    unsafe extern "system" fn window_procedure<F>(
        window: HWND,
        message: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT
    where
        F: Fn(UINT, WPARAM, LPARAM),
    {
        let raw_callback = GetWindowLongPtrW(window, GWLP_USERDATA);

        if raw_callback != 0 {
            let typed_callback = &mut *(raw_callback as *mut F);
            typed_callback(message, wparam, lparam);
        }

        if message == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        }

        DefWindowProcW(window, message, wparam, lparam)
    }
}

impl Drop for BroadcastListener {
    fn drop(&mut self) {
        unsafe {
            PostThreadMessageW(self.thread_id, REQUEST_THREAD_SHUTDOWN, 0, 0);
            WaitForSingleObject(self.thread_handle, INFINITE);
            CloseHandle(self.thread_handle);
        }
    }
}

pub type MonitorHandle = BroadcastListener;

pub fn spawn_monitor(sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    let listener =
        BroadcastListener::start(move |message: UINT, wparam: WPARAM, _lparam: LPARAM| {
            if message == WM_POWERBROADCAST {
                if wparam == PBT_APMSUSPEND {
                    debug!("Machine is preparing to enter sleep mode");
                    let _ = sender.unbounded_send(TunnelCommand::IsOffline(true));
                } else if wparam == PBT_APMRESUMEAUTOMATIC {
                    debug!("Machine is returning from sleep mode");
                    let cloned_sender = sender.clone();
                    thread::spawn(move || {
                        // TAP will be unavailable for approximately 2 seconds on a healthy machine.
                        thread::sleep(Duration::from_secs(5));
                        debug!("TAP is presumed to have been re-initialized");
                        let _ = cloned_sender.unbounded_send(TunnelCommand::IsOffline(false));
                    });
                }
            }
        })?;

    Ok(listener)
}

pub fn is_offline() -> bool {
    false
}
