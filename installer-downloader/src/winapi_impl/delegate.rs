//! This module implements [AppDelegate] and [Queue], which allows the NWG UI to be hooked up to our
//! generic controller.

use installer_downloader::delegate::ErrorMessage;
use native_windows_gui::{self as nwg, Event};
use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;

use super::ui::{AppWindow, QUEUE_MESSAGE};
use crate::delegate::{AppDelegate, AppDelegateQueue};

impl AppDelegate for AppWindow {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_click_handler(self.window.handle, self.download_button.handle, callback);
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_click_handler(self.window.handle, self.cancel_button.handle, callback);
    }

    fn on_beta_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_label_click_handler(self.window.handle, self.beta_link.handle, callback);
    }

    fn on_stable_link<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_frame_click_handler(self.stable_message_frame.handle, callback);
    }

    fn set_status_text(&mut self, text: &str) {
        self.status_text.set_visible(true);
        self.status_text.set_text(text);
    }

    fn clear_status_text(&mut self) {
        self.status_text.set_visible(false);
        self.status_text.set_text("");
    }

    fn set_download_text(&mut self, text: &str) {
        self.download_text.set_visible(true);
        self.download_text.set_text(text);
    }

    fn clear_download_text(&mut self) {
        self.download_text.set_visible(false);
        self.download_text.set_text("");
    }

    fn show_download_progress(&mut self) {
        self.progress_bar.set_visible(true);
    }

    fn hide_download_progress(&mut self) {
        self.progress_bar.set_visible(false);
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.progress_bar.set_pos(complete);
    }

    fn show_download_button(&mut self) {
        self.download_button.set_visible(true);
    }

    fn hide_download_button(&mut self) {
        self.download_button.set_visible(false);
    }

    fn enable_download_button(&mut self) {
        self.download_button.set_enabled(true);
    }

    fn disable_download_button(&mut self) {
        self.download_button.set_enabled(false);
    }

    fn show_cancel_button(&mut self) {
        self.cancel_button.set_visible(true);
    }

    fn hide_cancel_button(&mut self) {
        self.cancel_button.set_visible(false);
    }

    fn enable_cancel_button(&mut self) {
        self.cancel_button.set_enabled(true);
    }

    fn disable_cancel_button(&mut self) {
        self.cancel_button.set_enabled(false);
    }

    fn show_beta_text(&mut self) {
        self.beta_prefix.set_visible(true);
        self.beta_link.set_visible(true);
    }

    fn hide_beta_text(&mut self) {
        self.beta_prefix.set_visible(false);
        self.beta_link.set_visible(false);
    }

    fn show_stable_text(&mut self) {
        self.stable_message_frame.set_visible(true);
    }

    fn hide_stable_text(&mut self) {
        self.stable_message_frame.set_visible(false);
    }

    fn show_error_message(&mut self, error: ErrorMessage) {
        self.error_view.error_text.set_text(&error.status_text);
        self.error_view
            .error_retry_button
            .set_text(&error.retry_button_text);
        self.error_view
            .error_cancel_button
            .set_text(&error.cancel_button_text);

        self.error_view.error_frame.set_visible(true);
    }

    fn hide_error_message(&mut self) {
        self.error_view.error_frame.set_visible(false);
    }

    fn on_error_message_retry<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_click_handler(
            self.error_view.error_frame.handle,
            self.error_view.error_retry_button.handle,
            callback,
        );
    }

    fn on_error_message_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        register_click_handler(
            self.error_view.error_frame.handle,
            self.error_view.error_cancel_button.handle,
            callback,
        );
    }

    fn quit(&mut self) {
        nwg::stop_thread_dispatch();
    }

    fn queue(&self) -> Self::Queue {
        Queue {
            main_wnd: self.window.handle,
        }
    }
}

/// Register a window message for clicking this button that triggers `callback`.
fn register_click_handler(
    parent: nwg::ControlHandle,
    button: nwg::ControlHandle,
    callback: impl Fn() + 'static,
) {
    register_click_handler_inner(parent, button, callback, Event::OnButtonClick);
}

/// Register a window message for clicking this button that triggers `callback`.
fn register_label_click_handler(
    parent: nwg::ControlHandle,
    button: nwg::ControlHandle,
    callback: impl Fn() + 'static,
) {
    register_click_handler_inner(parent, button, callback, Event::OnLabelClick);
}

/// Register a window message for clicking this button that triggers `callback`.
fn register_click_handler_inner(
    parent: nwg::ControlHandle,
    button: nwg::ControlHandle,
    callback: impl Fn() + 'static,
    click_event: Event,
) {
    nwg::bind_event_handler(&button, &parent, move |evt, _, handle| {
        if evt == click_event && handle == button {
            callback();
        }
    });
}

/// Register a window message for clicking anything within a frame.
fn register_frame_click_handler(frame: nwg::ControlHandle, callback: impl Fn() + 'static) {
    nwg::bind_event_handler(&frame, &frame, move |evt, _, _handle| {
        if [Event::OnLabelClick, Event::OnImageFrameClick].contains(&evt) {
            callback();
        }
    });
}

/// Queue sends a window message to the main window containing a [QueueContext], giving us mutable
/// access to the [AppDelegate] on the main UI thread.
///
/// See [QueueContext] docs for more information.
#[derive(Clone)]
pub struct Queue {
    main_wnd: nwg::ControlHandle,
}

// SAFETY: It is safe to post window messages across threads
unsafe impl Send for Queue {}

/// The context contains a callback function that is passed as a pointer to the main thread
/// along with a custom window message `QUEUE_MESSAGE`.
///
/// It must be wrapped in a struct since we cannot pass a fat pointer
/// `*mut dyn for<'a> FnOnce(&'a mut AppWindow) + Send` to `PostMessageW`.
pub struct QueueContext {
    pub callback: Box<dyn for<'a> FnOnce(&'a mut AppWindow) + Send>,
}

impl AppDelegateQueue<AppWindow> for Queue {
    fn queue_main<F: FnOnce(&mut AppWindow) + 'static + Send>(&self, callback: F) {
        let Some(hwnd) = self.main_wnd.hwnd() else {
            return;
        };
        let context = QueueContext {
            callback: Box::new(callback),
        };
        let context_ptr = Box::into_raw(Box::new(context));
        // SAFETY: This is safe since `callback` is Send
        unsafe { PostMessageW(hwnd as _, QUEUE_MESSAGE, 0, context_ptr as isize) };
    }
}
