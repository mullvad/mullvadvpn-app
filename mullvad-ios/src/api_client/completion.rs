use std::sync::{Arc, Mutex};

use super::response::SwiftMullvadApiResponse;

extern "C" {
    /// Maps to `mullvadApiCompletionFinish` on Swift side to facilitate callback based completion flow when doing
    /// network calls through Mullvad API on Rust side.
    ///
    /// # Safety
    ///
    /// `response` must be pointing to a valid instance of `SwiftMullvadApiResponse`.
    ///
    /// `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is safe
    /// because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this type is
    /// intended to be used.
    pub fn mullvad_api_completion_finish(
        response: SwiftMullvadApiResponse,
        completion_cookie: CompletionCookie,
    );
}

#[repr(C)]
pub struct CompletionCookie(pub *mut std::ffi::c_void);
unsafe impl Send for CompletionCookie {}

#[derive(Clone)]
pub struct SwiftCompletionHandler {
    inner: Arc<Mutex<Option<CompletionCookie>>>,
}

impl SwiftCompletionHandler {
    pub fn new(cookie: CompletionCookie) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(cookie))),
        }
    }

    // This function makes sure that completion is done only once.
    pub fn finish(&self, response: SwiftMullvadApiResponse) {
        let Ok(mut maybe_cookie) = self.inner.lock() else {
            log::error!("Response handler panicked");
            return;
        };

        let Some(cookie) = maybe_cookie.take() else {
            return;
        };

        unsafe { mullvad_api_completion_finish(response, cookie) };
    }
}
