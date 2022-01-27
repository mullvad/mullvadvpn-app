//! Utilities for windows.

use std::{os::windows::io::AsRawHandle, ptr, thread};
use winapi::{
    shared::{
        basetsd::LONG_PTR,
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        libloaderapi::GetModuleHandleW,
        processthreadsapi::GetThreadId,
        winuser::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            GetWindowLongPtrW, PostQuitMessage, PostThreadMessageW, SetWindowLongPtrW,
            TranslateMessage, GWLP_USERDATA, GWLP_WNDPROC, WM_DESTROY, WM_USER,
        },
    },
};

const CLASS_NAME: &[u8] = b"S\0T\0A\0T\0I\0C\0\0\0";
const REQUEST_THREAD_SHUTDOWN: UINT = WM_USER + 1;

/// Handle for closing an associated window.
/// The window is not destroyed when this is dropped.
pub struct WindowCloseHandle {
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WindowCloseHandle {
    /// Close the window and wait for the thread.
    pub fn close(&mut self) {
        if let Some(thread) = self.thread.take() {
            let thread_id = unsafe { GetThreadId(thread.as_raw_handle()) };
            unsafe { PostThreadMessageW(thread_id, REQUEST_THREAD_SHUTDOWN, 0, 0) };
            let _ = thread.join();
        }
    }
}

/// Creates a dummy window whose messages are handled by `wnd_proc`.
pub fn create_hidden_window<F: (Fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) + Send + 'static>(
    wnd_proc: F,
) -> WindowCloseHandle {
    let join_handle = thread::spawn(move || {
        let dummy_window = unsafe {
            CreateWindowExW(
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
            )
        };

        // Move callback information to the heap.
        // This enables us to reach the callback through a "thin pointer".
        let raw_callback = Box::into_raw(Box::new(wnd_proc));

        unsafe {
            SetWindowLongPtrW(dummy_window, GWLP_USERDATA, raw_callback as LONG_PTR);
            SetWindowLongPtrW(
                dummy_window,
                GWLP_WNDPROC,
                window_procedure::<F> as LONG_PTR,
            );
        }

        let mut msg = unsafe { std::mem::zeroed() };

        loop {
            let status = unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };

            if status < 0 {
                continue;
            }
            if status == 0 {
                break;
            }

            if msg.hwnd.is_null() {
                if msg.message == REQUEST_THREAD_SHUTDOWN {
                    unsafe { DestroyWindow(dummy_window) };
                }
            } else {
                unsafe {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                }
            }
        }

        // Free callback.
        let _ = unsafe { Box::from_raw(raw_callback) };
    });

    WindowCloseHandle {
        thread: Some(join_handle),
    }
}

unsafe extern "system" fn window_procedure<F>(
    window: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    F: Fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
{
    if message == WM_DESTROY {
        PostQuitMessage(0);
        return 0;
    }
    let raw_callback = GetWindowLongPtrW(window, GWLP_USERDATA);
    if raw_callback != 0 {
        let typed_callback = &mut *(raw_callback as *mut F);
        return typed_callback(window, message, wparam, lparam);
    }
    DefWindowProcW(window, message, wparam, lparam)
}
