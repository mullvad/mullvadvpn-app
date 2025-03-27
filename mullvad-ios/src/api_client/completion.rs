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
pub struct CompletionCookie {
    inner: *mut std::ffi::c_void,
}
/// SAFETY: Access to `CompletionCookie` should always be done through a `SwiftCompletionHandler`
/// It is safe to be used and sent from any threads.
unsafe impl Send for CompletionCookie {}
impl CompletionCookie {
    /// `inner` must be pointing to a valid instance of Swift object `MullvadApiCompletion`.
    pub unsafe fn new(inner: *mut std::ffi::c_void) -> Self {
        Self { inner }
    }
}

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

        // SAFETY: See safety notes for `mullvad_api_completion_finish`
        unsafe { mullvad_api_completion_finish(response, cookie) };
    }
}
